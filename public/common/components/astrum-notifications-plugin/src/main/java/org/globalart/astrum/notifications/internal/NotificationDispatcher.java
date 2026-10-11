package org.globalart.astrum.notifications.internal;

import java.net.URLEncoder;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.concurrent.Executors;
import java.util.concurrent.ScheduledExecutorService;
import java.util.concurrent.ScheduledThreadPoolExecutor;
import java.util.concurrent.TimeUnit;
import java.util.function.LongSupplier;
import java.util.function.Supplier;
import java.util.stream.Collectors;

import javax.annotation.Nullable;

import org.globalart.astrum.common.QualifierUtil;
import org.globalart.astrum.common.app.BaseUrlHolder;
import org.globalart.astrum.common.app.BaseUrlManager;
import org.globalart.astrum.common.app.ManagedLifecycle;
import org.globalart.astrum.common.event.EventAware;
import org.globalart.astrum.common.stateguard.StateGuardLifecycleSupport;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.browse.node.BrowsePath;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.browse.BrowseNodeGenerator;
import org.globalart.astrum.repository.content.browse.DefaultBrowseNodeGenerator;
import org.globalart.astrum.repository.content.event.asset.AssetCreatedEvent;
import org.globalart.astrum.repository.content.event.asset.AssetEvent;
import org.globalart.astrum.repository.content.event.asset.AssetUploadedEvent;
import org.globalart.astrum.repository.content.event.component.ComponentCreatedEvent;
import org.globalart.astrum.repository.content.event.component.ComponentDeletedEvent;
import org.globalart.astrum.repository.content.event.component.ComponentsPurgedAuditEvent;
import org.globalart.astrum.repository.content.store.ContentStoreEvent;
import org.globalart.astrum.repository.content.store.InternalIds;
import org.globalart.astrum.repository.rest.api.AssetXODescriptor;
import org.globalart.astrum.repository.rest.api.RepositoryItemIDXO;
import org.globalart.astrum.repository.types.HostedType;
import org.globalart.astrum.security.UserIdHelper;
import org.globalart.astrum.thread.AstrumThreadFactory;

import com.google.common.annotations.VisibleForTesting;
import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;

import static com.google.common.base.Preconditions.checkArgument;
import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.base.Strings.isNullOrEmpty;
import static java.nio.charset.StandardCharsets.UTF_8;
import static org.globalart.astrum.common.app.ManagedLifecycle.Phase.TASKS;

/**
 * Turns component changes into notifications. A change is captured on the thread that made it, where the user and the
 * address of the server are known, and collected per channel; once a channel has been quiet for
 * {@code astrum.notifications.debounceSeconds}, the collected changes are sent in the background, so that one
 * deployment of several files gives one message per component.
 *
 * <p>
 * A component is published when it is created, updated when files are stored into an existing component, and deleted
 * when it is deleted or purged. Channels without repositories follow the hosted repositories.
 */
@org.springframework.stereotype.Component
@ManagedLifecycle(phase = TASKS)
public class NotificationDispatcher
    extends StateGuardLifecycleSupport
    implements EventAware
{
  static final int MAX_ATTEMPTS = 4;

  static final int BUFFER_CAPACITY = 500;

  private static final long DEFAULT_BACKOFF_MILLIS = 2000;

  private final NotificationChannelStore store;

  private final NotificationTransport transport;

  private final BaseUrlManager baseUrlManager;

  private final Map<String, BrowseNodeGenerator> browseNodeGenerators;

  private final Map<String, AssetXODescriptor> assetDescriptors;

  private final NotificationBuffer buffer;

  private final LongSupplier clock;

  private final Supplier<String> initiator;

  private final long backoffMillis;

  private volatile boolean running;

  private volatile ScheduledExecutorService flusher;

  private volatile ScheduledExecutorService sender;

  private volatile NotificationDelivery delivery;

  @Autowired
  public NotificationDispatcher(
      final NotificationChannelStore store,
      final NotificationTransport transport,
      final BaseUrlManager baseUrlManager,
      final List<BrowseNodeGenerator> browseNodeGenerators,
      final List<AssetXODescriptor> assetDescriptors,
      @Value("${astrum.notifications.debounceSeconds:10}") final int debounceSeconds,
      @Value("${astrum.notifications.maxDelaySeconds:60}") final int maxDelaySeconds)
  {
    this(store, transport, baseUrlManager, browseNodeGenerators, assetDescriptors,
        new NotificationBuffer(seconds(debounceSeconds), Math.max(seconds(debounceSeconds), seconds(maxDelaySeconds)),
            BUFFER_CAPACITY),
        System::currentTimeMillis, UserIdHelper::get, DEFAULT_BACKOFF_MILLIS);
  }

  @VisibleForTesting
  NotificationDispatcher(
      final NotificationChannelStore store,
      final NotificationTransport transport,
      final BaseUrlManager baseUrlManager,
      final List<BrowseNodeGenerator> browseNodeGenerators,
      final List<AssetXODescriptor> assetDescriptors,
      final NotificationBuffer buffer,
      final LongSupplier clock,
      final Supplier<String> initiator,
      final long backoffMillis)
  {
    this.store = checkNotNull(store);
    this.transport = checkNotNull(transport);
    this.baseUrlManager = checkNotNull(baseUrlManager);
    this.browseNodeGenerators = QualifierUtil.buildQualifierBeanMap(checkNotNull(browseNodeGenerators));
    this.assetDescriptors = QualifierUtil.buildQualifierBeanMap(checkNotNull(assetDescriptors));
    this.buffer = checkNotNull(buffer);
    this.clock = checkNotNull(clock);
    this.initiator = checkNotNull(initiator);
    this.backoffMillis = backoffMillis;
  }

  private static long seconds(final int seconds) {
    checkArgument(seconds >= 0, "Notification delays must not be negative");
    return TimeUnit.SECONDS.toMillis(seconds);
  }

  @Override
  protected void doStart() {
    ScheduledThreadPoolExecutor senderPool =
        new ScheduledThreadPoolExecutor(2, new AstrumThreadFactory("notifications", "sender"));
    senderPool.setExecuteExistingDelayedTasksAfterShutdownPolicy(false);
    sender = senderPool;
    delivery = new NotificationDelivery(store, transport, senderPool, backoffMillis, MAX_ATTEMPTS);
    flusher = Executors.newSingleThreadScheduledExecutor(new AstrumThreadFactory("notifications", "flusher"));
    flusher.scheduleWithFixedDelay(this::flushReady, 1, 1, TimeUnit.SECONDS);
    running = true;
  }

  @Override
  protected void doStop() throws InterruptedException {
    running = false;
    ScheduledExecutorService currentFlusher = flusher;
    if (currentFlusher != null) {
      currentFlusher.shutdownNow();
    }
    deliver(buffer.drainAll());
    ScheduledExecutorService currentSender = sender;
    if (currentSender != null) {
      currentSender.shutdown();
      if (!currentSender.awaitTermination(10, TimeUnit.SECONDS)) {
        currentSender.shutdownNow();
      }
    }
    delivery = null;
  }

  @Subscribe
  @AllowConcurrentEvents
  public void on(final ComponentCreatedEvent event) {
    capture(event, event.getComponent(), NoticeAction.PUBLISHED, null);
  }

  @Subscribe
  @AllowConcurrentEvents
  public void on(final ComponentDeletedEvent event) {
    capture(event, event.getComponent(), NoticeAction.DELETED, null);
  }

  @Subscribe
  @AllowConcurrentEvents
  public void on(final ComponentsPurgedAuditEvent event) {
    if (!running || store.enabled().isEmpty()) {
      return;
    }
    for (Component component : event.getComponents()) {
      capture(event, component, NoticeAction.DELETED, null);
    }
  }

  @Subscribe
  @AllowConcurrentEvents
  public void on(final AssetCreatedEvent event) {
    captureAsset(event);
  }

  @Subscribe
  @AllowConcurrentEvents
  public void on(final AssetUploadedEvent event) {
    captureAsset(event);
  }

  /**
   * Sends the changes of the channels that have been quiet long enough.
   */
  @VisibleForTesting
  void flushReady() {
    try {
      deliver(buffer.drainReady(clock.getAsLong()));
    }
    catch (RuntimeException e) {
      log.warn("Could not send the collected notifications", e);
    }
  }

  private void captureAsset(final AssetEvent event) {
    Asset asset = event.getAsset();
    Optional<Component> component = asset.component();
    if (component.isPresent()) {
      capture(event, component.get(), NoticeAction.UPDATED, asset);
    }
  }

  private void capture(
      final ContentStoreEvent event,
      final Component component,
      final NoticeAction action,
      @Nullable final Asset asset)
  {
    if (!running) {
      return;
    }
    try {
      List<NotificationChannel> channels = store.enabled();
      if (channels.isEmpty()) {
        return;
      }
      Optional<Repository> found = event.getRepository();
      if (found.isEmpty()) {
        return;
      }
      Repository repository = found.get();
      List<NotificationChannel> matching = channels.stream().filter(channel -> follows(channel, repository)).toList();
      if (matching.isEmpty()) {
        return;
      }
      String format = repository.getFormat().getValue();
      ComponentNotice notice = new ComponentNotice(
          repository.getName(),
          format,
          component.namespace(),
          component.name(),
          component.version(),
          action,
          initiator.get(),
          browsePath(format, asset),
          registry(repository, format, action),
          baseUrl(),
          repository.getType().getValue(),
          componentId(repository, component));
      long now = clock.getAsLong();
      for (NotificationChannel channel : matching) {
        buffer.add(channel.getId(), notice, now);
      }
    }
    catch (RuntimeException e) {
      log.debug("Could not capture a notification for {}", component, e);
    }
  }

  private static boolean follows(final NotificationChannel channel, final Repository repository) {
    if (channel.getRepositories().isEmpty()) {
      return HostedType.NAME.equals(repository.getType().getValue());
    }
    return channel.getRepositories().contains(repository.getName());
  }

  private void deliver(final Map<String, NotificationBuffer.Batch> batches) {
    NotificationDelivery current = delivery;
    if (current == null || batches.isEmpty()) {
      return;
    }
    batches.forEach((channelId, batch) -> {
      Optional<NotificationChannel> channel = store.find(channelId).filter(NotificationChannel::isEnabled);
      if (channel.isEmpty()) {
        return;
      }
      List<ComponentNotice> wanted = batch.notices().stream()
          .filter(notice -> channel.get().wants(notice.action()))
          .toList();
      if (wanted.isEmpty()) {
        return;
      }
      if (channel.get().isWebhook() && batch.overflow() > 0) {
        log.warn("Dropped {} component changes for channel {}: more than {} in one window", batch.overflow(),
            channel.get().getName(), BUFFER_CAPACITY);
      }
      for (NotificationMessage message : NotificationMessages.render(channel.get(), wanted, batch.overflow())) {
        current.submit(channelId, message);
      }
    });
  }

  /**
   * @return the component's id in the REST API, or {@code null} when the component has no id
   */
  @Nullable
  private static String componentId(final Repository repository, final Component component) {
    try {
      String id = InternalIds.toExternalId(InternalIds.internalComponentId(component)).getValue();
      return new RepositoryItemIDXO(repository.getName(), id).getValue();
    }
    catch (RuntimeException e) {
      return null;
    }
  }

  /**
   * @return the component's node in the browse tree, its segments encoded like the browse tree's node ids
   */
  @Nullable
  private String browsePath(final String format, @Nullable final Asset asset) {
    if (asset == null || asset.component().isEmpty()) {
      return null;
    }
    BrowseNodeGenerator generator = browseNodeGenerators.get(format);
    if (generator == null) {
      generator = browseNodeGenerators.get(DefaultBrowseNodeGenerator.NAME);
    }
    if (generator == null) {
      return null;
    }
    List<BrowsePath> paths = generator.computeComponentPaths(asset);
    if (paths.isEmpty()) {
      return null;
    }
    return paths.stream()
        .map(path -> URLEncoder.encode(path.getDisplayName(), UTF_8))
        .collect(Collectors.joining("/"));
  }

  @Nullable
  private String registry(final Repository repository, final String format, final NoticeAction action) {
    if (action == NoticeAction.DELETED) {
      return null;
    }
    AssetXODescriptor descriptor = assetDescriptors.get(format);
    if (descriptor == null) {
      return null;
    }
    String configured = baseUrlManager.getUrl();
    if (isNullOrEmpty(configured)) {
      return BaseUrlHolder.isSet() ? descriptor.computeRegistryUrl(repository) : null;
    }
    String previousUrl = BaseUrlHolder.isSet() ? BaseUrlHolder.get() : null;
    String previousRelativePath = previousUrl != null ? BaseUrlHolder.getRelativePath() : null;
    BaseUrlHolder.set(configured, "");
    try {
      return descriptor.computeRegistryUrl(repository);
    }
    finally {
      if (previousUrl != null) {
        BaseUrlHolder.set(previousUrl, previousRelativePath);
      }
      else {
        BaseUrlHolder.unset();
      }
    }
  }

  @Nullable
  private String baseUrl() {
    String configured = baseUrlManager.getUrl();
    if (!isNullOrEmpty(configured)) {
      return configured;
    }
    return BaseUrlHolder.isSet() ? BaseUrlHolder.get() : null;
  }
}
