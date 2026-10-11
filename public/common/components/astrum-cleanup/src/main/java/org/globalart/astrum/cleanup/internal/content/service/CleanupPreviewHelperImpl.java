
package org.globalart.astrum.cleanup.internal.content.service;

import java.time.Duration;
import java.util.Collection;
import java.util.List;
import java.util.Objects;
import java.util.Set;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.Future;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.function.Predicate;
import java.util.stream.Collectors;
import java.util.stream.Stream;

import javax.annotation.Nullable;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.WebApplicationException;
import jakarta.ws.rs.core.Response;

import org.globalart.astrum.cleanup.content.search.CleanupBrowseServiceFactory;
import org.globalart.astrum.cleanup.content.search.CleanupComponentBrowse;
import org.globalart.astrum.cleanup.preview.CleanupPreviewHelper;
import org.globalart.astrum.cleanup.preview.CleanupPreviewSummaryXO;
import org.globalart.astrum.cleanup.storage.CleanupPolicy;
import org.globalart.astrum.cleanup.storage.CleanupPolicyCriteria;
import org.globalart.astrum.cleanup.storage.CleanupPolicyPreviewXO;
import org.globalart.astrum.cleanup.storage.CleanupPolicyStorage;
import org.globalart.astrum.extdirect.model.PagedResponse;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.AssetBlob;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.query.QueryOptions;
import org.globalart.astrum.repository.rest.api.AssetXO;
import org.globalart.astrum.repository.rest.api.ComponentXO;
import org.globalart.astrum.repository.rest.api.DefaultComponentXO;
import org.globalart.astrum.rest.ValidationErrorsException;
import org.globalart.astrum.scheduling.CancelableHelper;
import org.globalart.astrum.scheduling.TaskInterruptedException;

import org.globalart.astrum.common.time.DateHelper;
import org.springframework.beans.factory.annotation.Value;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.app.FeatureFlags.CLEANUP_RETAIN_ALL_FORMATS_NAMED_VALUE;
import static org.globalart.astrum.common.time.DateHelper.optionalOffsetToDate;

/**
 * {@link CleanupPreviewHelper} implementation.
 */
@org.springframework.stereotype.Component
public class CleanupPreviewHelperImpl
    implements CleanupPreviewHelper
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final CleanupPolicyStorage cleanupPolicyStorage;

  private final Duration previewTimeout;

  private final Duration summaryTimeout;

  private final CleanupBrowseServiceFactory browseServiceFactory;

  private final boolean retainAllFormatsEnabled;

  @Autowired
  public CleanupPreviewHelperImpl(
      final CleanupPolicyStorage cleanupPolicyStorage,
      @Value("${astrum.cleanup.preview.timeout:60s}") final Duration previewTimeout,
      @Value("${astrum.cleanup.preview.summary.timeout:120s}") final Duration summaryTimeout,
      final CleanupBrowseServiceFactory browseServiceFactory,
      @Value(CLEANUP_RETAIN_ALL_FORMATS_NAMED_VALUE) final boolean retainAllFormatsEnabled)
  {
    this.cleanupPolicyStorage = checkNotNull(cleanupPolicyStorage);
    this.previewTimeout = checkNotNull(previewTimeout);
    this.summaryTimeout = checkNotNull(summaryTimeout);
    this.browseServiceFactory = checkNotNull(browseServiceFactory);
    this.retainAllFormatsEnabled = retainAllFormatsEnabled;
  }

  @Override
  public PagedResponse<ComponentXO> getSearchResults(
      final CleanupPolicyPreviewXO previewXO,
      final Repository repository,
      final QueryOptions queryOptions)
  {
    CleanupPolicy cleanupPolicy = toCleanupPolicy(previewXO);

    return searchForComponents(repository, cleanupPolicy, queryOptions);
  }

  @Override
  public Stream<ComponentXO> getSearchResultsStream(
      final CleanupPolicyPreviewXO previewXO,
      final Repository repository,
      final QueryOptions queryOptions)
  {
    CleanupPolicy cleanupPolicy = toCleanupPolicy(previewXO);

    CleanupComponentBrowse browseService = browseServiceFactory.getPreviewService();
    Stream<FluentComponent> componentSteam =
        browseService.browseIncludingAssets(cleanupPolicy, repository);

    Stream<ComponentXO> xoStream = componentSteam.map(component -> convert(component, repository));
    return retainAllFormatsEnabled ? dedupeByGroupNameVersion(xoStream) : xoStream;
  }

  private PagedResponse<ComponentXO> searchForComponents(
      final Repository repository,
      final CleanupPolicy cleanupPolicy,
      final QueryOptions queryOptions)
  {
    PagedResponse<Component> components = browse(cleanupPolicy, repository, queryOptions);

    Stream<ComponentXO> xoStream = components.getData().stream().map(item -> convert(item, repository));
    List<ComponentXO> componentXOS =
        (retainAllFormatsEnabled ? dedupeByGroupNameVersion(xoStream) : xoStream)
            .collect(Collectors.toUnmodifiableList());

    return new PagedResponse<>(components.getTotal(), componentXOS);
  }

  /**
   * Filters a stream of {@link ComponentXO} keeping the first occurrence of each {@code (group, name, version)} tuple
   * and preserving original ordering. Used to collapse duplicate component rows produced by JOIN-multiplication in the
   * preview SQL without buffering the full result set in memory.
   */
  private static Stream<ComponentXO> dedupeByGroupNameVersion(final Stream<ComponentXO> stream) {
    Set<String> seen = ConcurrentHashMap.newKeySet();
    return stream.filter(xo -> seen.add(dedupeKey(xo)));
  }

  private static String dedupeKey(final ComponentXO xo) {
    return (xo.getGroup() == null ? "" : xo.getGroup()) + '\u0000' +
        (xo.getName() == null ? "" : xo.getName()) + '\u0000' +
        (xo.getVersion() == null ? "" : xo.getVersion());
  }

  private PagedResponse<Component> browse(
      final CleanupPolicy policy,
      final Repository repository,
      final QueryOptions queryOptions)
  {
    AtomicBoolean cancelled = new AtomicBoolean(false);

    Future<PagedResponse<Component>> future = CompletableFuture.supplyAsync(() -> {
      CancelableHelper.set(cancelled);
      try {
        // compute preview and return it
        CleanupComponentBrowse browseService = browseServiceFactory.getPreviewService();
        return browseService.browseByPage(policy, repository, queryOptions);
      }
      finally {
        CancelableHelper.remove();
      }
    });

    try {
      return future.get(previewTimeout.toMillis(), TimeUnit.MILLISECONDS);
    }
    catch (TimeoutException e) {
      cancelled.set(true);
      log.debug("Timeout computing preview for cleanup for policy {} in repository {}", policy, repository.getName(),
          e);
      // indicate failure to UI
      throw new WebApplicationException(
          Response.serverError().entity("A timeout occurred while computing the preview results.").build());
    }
    catch (ExecutionException e) {
      throw failure(e);
    }
    catch (InterruptedException e) {
      cancelled.set(true);
      Thread.currentThread().interrupt();
      throw new WebApplicationException(e, Response.serverError().build());
    }
  }

  @Override
  public CleanupPreviewSummaryXO getSummary(
      final CleanupPolicyPreviewXO previewXO,
      final Repository repository,
      @Nullable final String filter)
  {
    CleanupPolicy policy = toCleanupPolicy(previewXO);

    if (policy.getCriteria().isEmpty()) {
      return new CleanupPreviewSummaryXO(repository.getName(), 0, 0, 0, true);
    }

    Predicate<FluentComponent> textFilter = textFilter(filter);
    Totals totals = new Totals();
    AtomicBoolean cancelled = new AtomicBoolean(false);

    Future<Void> future = CompletableFuture.runAsync(() -> {
      CancelableHelper.set(cancelled);
      try (Stream<FluentComponent> components =
               browseServiceFactory.getPreviewService().browseIncludingAssets(policy, repository)) {
        components
            .peek(__ -> CancelableHelper.checkCancellation())
            .filter(textFilter)
            .forEach(totals::add);
      }
      catch (TaskInterruptedException e) {
        if (!cancelled.get()) {
          throw e;
        }
        log.debug("Stopped summing up cleanup preview of repository {}", repository.getName());
      }
      finally {
        CancelableHelper.remove();
      }
    });

    try {
      future.get(summaryTimeout.toMillis(), TimeUnit.MILLISECONDS);
      return totals.toXO(repository.getName(), true);
    }
    catch (TimeoutException e) {
      cancelled.set(true);
      CleanupPreviewSummaryXO partial = totals.toXO(repository.getName(), false);
      log.debug("Cleanup preview of repository {} reached its time limit of {}: {}", repository.getName(),
          summaryTimeout, partial);
      return partial;
    }
    catch (ExecutionException e) {
      throw failure(e);
    }
    catch (InterruptedException e) {
      cancelled.set(true);
      Thread.currentThread().interrupt();
      throw new WebApplicationException(e, Response.serverError().build());
    }
  }

  /**
   * Unwraps what the background computation threw. Criteria without an evaluator are a mistake in the request rather
   * than a server failure, so they are reported as a validation error.
   */
  private static RuntimeException failure(final ExecutionException e) {
    Throwable cause = e.getCause();
    if (cause instanceof UnsupportedOperationException) {
      return new ValidationErrorsException(
          "The preview cannot evaluate this policy: " + cause.getMessage() +
              ". Remove the unsupported criteria to see what the policy would delete.", cause);
    }
    if (cause instanceof RuntimeException runtimeException) {
      return runtimeException;
    }
    return new WebApplicationException(cause, Response.serverError().build());
  }

  private static Predicate<FluentComponent> textFilter(@Nullable final String filter) {
    if (filter == null || filter.isBlank()) {
      return component -> true;
    }
    return component -> Objects.toString(component.name(), "").contains(filter) ||
        Objects.toString(component.namespace(), "").contains(filter) ||
        Objects.toString(component.version(), "").contains(filter);
  }

  /**
   * Running totals shared with the request thread, which reads them when the time limit is reached.
   */
  private static final class Totals
  {
    private long components;

    private long assets;

    private long size;

    void add(final FluentComponent component) {
      Collection<FluentAsset> componentAssets = component.assets();
      long componentSize = componentAssets.stream().mapToLong(FluentAsset::assetBlobSize).sum();
      synchronized (this) {
        components++;
        assets += componentAssets.size();
        size += componentSize;
      }
    }

    synchronized CleanupPreviewSummaryXO toXO(final String repository, final boolean complete) {
      return new CleanupPreviewSummaryXO(repository, components, assets, size, complete);
    }
  }

  private CleanupPolicy toCleanupPolicy(final CleanupPolicyPreviewXO cleanupPolicyPreviewXO) {
    CleanupPolicy policy = cleanupPolicyStorage.newCleanupPolicy();

    policy.setName("preview");
    policy.setCriteria(CleanupPolicyCriteria.toMap(cleanupPolicyPreviewXO.getCriteria()));

    return policy;
  }

  private static ComponentXO convert(final Component component, final Repository repository) {
    DefaultComponentXO defaultComponentXO = new DefaultComponentXO();
    defaultComponentXO.setRepository(repository.getName());
    defaultComponentXO.setGroup(component.namespace());
    defaultComponentXO.setName(component.name());
    defaultComponentXO.setVersion(component.version());
    defaultComponentXO.setFormat(repository.getFormat().getValue());
    return defaultComponentXO;
  }

  private static ComponentXO convert(final FluentComponent component, final Repository repository) {
    ComponentXO componentXO = convert((Component) component, repository);

    List<AssetXO> assetXOS = convert(component.assets(true));

    componentXO.setAssets(assetXOS);

    return componentXO;
  }

  private static List<AssetXO> convert(final Collection<FluentAsset> assets) {
    return assets
        .stream()
        .map(it -> {
          AssetXO assetXO = new AssetXO();

          assetXO.setPath(it.path());
          assetXO.setBlobStoreName(it.blobStoreName());
          assetXO.setFileSize(it.assetBlobSize());
          assetXO.setLastDownloaded(optionalOffsetToDate(it.lastDownloaded()));
          assetXO.setBlobCreated(it.blob().map(AssetBlob::blobCreated).map(DateHelper::offsetToDate).orElse(null));

          return assetXO;
        })
        .collect(Collectors.toList());
  }
}
