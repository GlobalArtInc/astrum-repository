package org.globalart.astrum.repository.yum.internal.task;

import java.time.Duration;
import java.util.Date;
import java.util.Map;
import java.util.Objects;
import java.util.Optional;

import javax.annotation.Nullable;

import org.globalart.astrum.common.cooperation2.Cooperation2;
import org.globalart.astrum.common.cooperation2.Cooperation2Factory;
import org.globalart.astrum.common.event.EventAware;
import org.globalart.astrum.common.time.Clock;
import org.globalart.astrum.repository.Facet.Exposed;
import org.globalart.astrum.repository.FacetSupport;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.RepositoryTaskSupport;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.event.asset.AssetCreatedEvent;
import org.globalart.astrum.repository.content.event.asset.AssetDeletedEvent;
import org.globalart.astrum.repository.content.event.asset.AssetPurgedEvent;
import org.globalart.astrum.repository.content.event.asset.AssetUpdatedEvent;
import org.globalart.astrum.repository.content.event.component.ComponentPurgedEvent;
import org.globalart.astrum.repository.content.store.ContentStoreEvent;
import org.globalart.astrum.repository.manager.RepositoryUpdatedEvent;
import org.globalart.astrum.repository.yum.AssetKind;
import org.globalart.astrum.repository.yum.internal.YumSigningFacet;
import org.globalart.astrum.repository.yum.internal.hosted.YumHostedFacet;
import org.globalart.astrum.scheduling.TaskConfiguration;
import org.globalart.astrum.scheduling.TaskInfo;
import org.globalart.astrum.scheduling.TaskScheduler;
import org.globalart.astrum.scheduling.schedule.Once;

import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkArgument;
import static com.google.common.base.Preconditions.checkNotNull;

@Component
@Exposed
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class YumMetadataRebuildFacet
    extends FacetSupport
    implements EventAware.Asynchronous
{
  private final TaskScheduler taskScheduler;

  private final Cooperation2Factory.Builder cooperationBuilder;

  private final Duration rebuildDelay;

  private final Clock clock;

  private Cooperation2 cooperation;

  @Autowired
  public YumMetadataRebuildFacet(
      final TaskScheduler taskScheduler,
      final Cooperation2Factory cooperationFactory,
      final Clock clock,
      @Value("${astrum.yum.metadata.rebuild.debounce:2s}") final Duration rebuildDelay,
      @Value("${astrum.yum.metadata.cooperation.enabled:true}") final boolean cooperationEnabled,
      @Value("${astrum.yum.metadata.cooperation.majorTimeout:0s}") final Duration majorTimeout,
      @Value("${astrum.yum.metadata.cooperation.minorTimeout:30s}") final Duration minorTimeout,
      @Value("${astrum.yum.metadata.cooperation.threadsPerKey:100}") final int threadsPerKey)
  {
    this.taskScheduler = checkNotNull(taskScheduler);
    this.clock = checkNotNull(clock);
    checkArgument(!rebuildDelay.isNegative(), "astrum.yum.metadata.rebuild.debounce must not be negative");
    this.rebuildDelay = rebuildDelay;
    this.cooperationBuilder = checkNotNull(cooperationFactory).configure()
        .enabled(cooperationEnabled)
        .majorTimeout(majorTimeout)
        .minorTimeout(minorTimeout)
        .threadsPerKey(threadsPerKey);
  }

  @Override
  protected void doStart() {
    cooperation = cooperationBuilder.build(getClass(), getRepository().getName());
  }

  @Subscribe
  @AllowConcurrentEvents
  public void on(final AssetCreatedEvent event) {
    onAsset(event, event.getAsset());
  }

  @Subscribe
  @AllowConcurrentEvents
  public void on(final AssetUpdatedEvent event) {
    onAsset(event, event.getAsset());
  }

  @Subscribe
  @AllowConcurrentEvents
  public void on(final AssetDeletedEvent event) {
    onAsset(event, event.getAsset());
  }

  @Subscribe
  @AllowConcurrentEvents
  public void on(final AssetPurgedEvent event) {
    onAsset(event, null);
  }

  @Subscribe
  @AllowConcurrentEvents
  public void on(final ComponentPurgedEvent event) {
    onAsset(event, null);
  }

  @Subscribe
  @AllowConcurrentEvents
  public void on(final RepositoryUpdatedEvent event) {
    Repository repository = event.getRepository();
    if (!isThisRepository(repository)) {
      return;
    }
    Configuration previous = event.getOldConfiguration();
    Configuration current = repository.getConfiguration();
    if (!Objects.equals(section(previous, YumHostedFacet.CONFIG_KEY), section(current, YumHostedFacet.CONFIG_KEY))
        || !Objects.equals(keypair(previous), keypair(current))) {
      maybeScheduleRebuild();
    }
  }

  private void onAsset(final ContentStoreEvent event, @Nullable final Asset asset) {
    Optional<Repository> repository = event.getRepository();
    if (repository.isPresent() && isThisRepository(repository.get())
        && (asset == null || AssetKind.RPM.name().equals(asset.kind()))) {
      maybeScheduleRebuild();
    }
  }

  public void maybeScheduleRebuild() {
    String repositoryName = getRepository().getName();
    try {
      TaskInfo taskInfo = cooperation.on(this::scheduleRebuild)
          .checkFunction(this::scheduledTask)
          .performWorkOnFail(false)
          .cooperate(repositoryName);
      log.debug("Metadata rebuild of repository {} is scheduled as {}", repositoryName, taskInfo);
    }
    catch (Exception e) {
      log.warn("Failed to schedule the metadata rebuild of repository {}: {}", repositoryName, e.getMessage(),
          log.isDebugEnabled() ? e : null);
    }
  }

  private TaskInfo scheduleRebuild() {
    Optional<TaskInfo> waiting = scheduledTask();
    if (waiting.isPresent()) {
      return waiting.get();
    }
    String repositoryName = getRepository().getName();
    TaskConfiguration configuration =
        taskScheduler.createTaskConfigurationInstance(AutomatedYumMetadataRebuildTaskDescriptor.TYPE_ID);
    configuration.setName("Yum metadata rebuild for " + repositoryName);
    configuration.setString(RepositoryTaskSupport.REPOSITORY_NAME_FIELD_ID, repositoryName);
    long time = clock.clusterTime().plus(rebuildDelay).toInstant().toEpochMilli();
    return taskScheduler.scheduleTask(configuration, new Once(new Date(time)));
  }

  private Optional<TaskInfo> scheduledTask() {
    return taskScheduler.listsTasks()
        .stream()
        .filter(task -> AutomatedYumMetadataRebuildTaskDescriptor.TYPE_ID.equals(task.getTypeId()))
        .filter(task -> getRepository().getName()
            .equals(task.getConfiguration().getString(RepositoryTaskSupport.REPOSITORY_NAME_FIELD_ID)))
        .filter(task -> taskScheduler.toExternalTaskState(task).getState().isWaiting())
        .findAny();
  }

  private boolean isThisRepository(final Repository repository) {
    return getRepository().getName().equals(repository.getName());
  }

  @Nullable
  private static Object section(@Nullable final Configuration configuration, final String key) {
    return configuration == null || configuration.getAttributes() == null
        ? null : configuration.getAttributes().get(key);
  }

  @Nullable
  private static Object keypair(@Nullable final Configuration configuration) {
    Object signing = section(configuration, YumSigningFacet.CONFIG_KEY);
    return signing instanceof Map<?, ?> map ? map.get(YumSigningFacet.KEYPAIR) : null;
  }
}
