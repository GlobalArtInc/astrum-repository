
package org.globalart.astrum.repository.search.sql.index;

import java.util.Set;
import java.util.concurrent.TimeUnit;

import org.globalart.astrum.repository.search.sql.store.SearchStore;
import org.globalart.astrum.common.app.ManagedLifecycle;
import org.globalart.astrum.common.event.EventAware;
import org.globalart.astrum.common.scheduling.PeriodicJobService;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.event.component.ComponentCreatedEvent;
import org.globalart.astrum.repository.content.event.component.ComponentEvent;
import org.globalart.astrum.repository.content.event.repository.ContentRepositoryDeletedEvent;
import org.globalart.astrum.repository.content.search.SearchEventHandler;
import org.globalart.astrum.repository.content.search.SearchFacet;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.types.ProxyType;

import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.app.ManagedLifecycle.Phase.SERVICES;
import static org.globalart.astrum.repository.content.store.InternalIds.contentRepositoryId;
import static org.globalart.astrum.repository.content.store.InternalIds.toExternalId;

/**
 * <ul>
 * <li>Deletes all search indexes for a given content repository in response to a ContentRepositoryDeletedEvent</li>
 * <li>Skips ComponentCreatedEvents</li>
 * </ul>
 */
@ManagedLifecycle(phase = SERVICES)
@Component
public class SqlSearchEventHandler
    extends SearchEventHandler
    implements EventAware
{
  private final SearchStore searchTableStore;

  @Autowired
  public SqlSearchEventHandler(
      final SearchStore searchTableStore,
      final RepositoryManager repositoryManager,
      final PeriodicJobService periodicJobService,
      @Value("${" + FLUSH_ON_COUNT_KEY + ":100}") final int flushOnCount,
      @Value("${" + FLUSH_ON_SECONDS_KEY + ":2}") final int flushOnSeconds,
      @Value("${" + NO_PURGE_DELAY_KEY + ":true}") final boolean noPurgeDelay,
      @Value("${" + FLUSH_POOL_SIZE + ":128}") final int poolSize)
  {
    super(repositoryManager, periodicJobService, flushOnCount, flushOnSeconds, noPurgeDelay, poolSize);
    this.searchTableStore = checkNotNull(searchTableStore);
  }

  @Override
  protected String getThreadPoolId() {
    return "sqlSearchEventHandler";
  }

  @Override
  protected void requestIndex(final ComponentEvent event) {
    if (event instanceof ComponentCreatedEvent) {
      log.debug("Skipping component created event for {}:{}:{}",
          event.getComponent().namespace(),
          event.getComponent().name(),
          event.getComponent().version());
      return;
    }
    super.requestIndex(event);
  }

  /** Proxy repositories write immediately; hosted and group use the base-class batch path. */
  @Override
  public void requestIndex(final String format, final int componentId, final Repository repository) {
    if (componentId <= 0 || !isProcessEvents()) {
      return;
    }
    if (isProxyRepository(repository)) {
      threadPoolExecutor.execute(() -> repository.optionalFacet(SearchFacet.class)
          .ifPresent(searchFacet -> {
            try {
              searchFacet.index(Set.of(toExternalId(componentId)));
            }
            catch (IllegalStateException e) {
              // repository stopped/deleted between optionalFacet() and index() — swallow safely.
              log.debug("Skipping index for stopped/deleted repository {}: {}", repository.getName(), e.getMessage());
            }
          }));
    }
    else {
      // Hosted/group: batch path is sufficient — no HA race concern.
      super.requestIndex(format, componentId, repository);
    }
  }

  /**
   * Extends base flush to also drain the thread pool used by proxy repos.
   * Proxy repos bypass pendingRequests, so the base flush is a no-op for them.
   * A sentinel task guarantees all prior index tasks have completed before returning.
   */
  @Override
  public void flushPendingForRepository(final String repositoryName) {
    super.flushPendingForRepository(repositoryName);
    try {
      // Sentinel drains all proxy tasks in the shared pool, not only this repository's.
      // Acceptable tradeoff: flush is cold-cache-only; per-repo tracking adds significant complexity.
      threadPoolExecutor.submit(() -> {
      }).get(5, TimeUnit.SECONDS);
    }
    catch (InterruptedException e) {
      Thread.currentThread().interrupt();
    }
    catch (Exception e) {
      log.warn("Timed out waiting for search index flush for repository {}", repositoryName);
    }
  }

  private static boolean isProxyRepository(final Repository repository) {
    return repository.getType() != null && ProxyType.NAME.equals(repository.getType().getValue());
  }

  @AllowConcurrentEvents
  @Subscribe
  public void on(final ContentRepositoryDeletedEvent event) {
    // As we are operating on a content repository basis rather than components, we can schedule immediately
    threadPoolExecutor.execute(() -> deleteRepositoryComponents(event));
    threadPoolExecutor.execute(() -> deleteSearchAssetRecords(event));
  }

  private boolean deleteRepositoryComponents(final ContentRepositoryDeletedEvent event) {
    int repositoryId = contentRepositoryId(event);
    String format = event.getFormat();
    log.debug("Deleting repository id: {}, format: {} from component_search", repositoryId, format);

    // Delete all records for repository - we don't need to use cooperation.
    // Note: In the SearchTableDao.save SQL we use an INSERT FROM SELECT and EXISTS check via the format_component table
    // to check that a component still exists before saving the record. Thus, if we have a scenario where we have
    // already purged all component search records associated with this repository and another node then
    // processes a delayed re-indexing of a component in the component search table
    // (e.g. in response to a AssetEvent, ComponentEvent which occurred before the ContentRepositoryDeletedEvent), that
    // update will, as expected, not re-insert the record because the component no longer exists.
    return searchTableStore.deleteAllForRepository(repositoryId, format);
  }

  private boolean deleteSearchAssetRecords(final ContentRepositoryDeletedEvent event) {
    int repositoryId = contentRepositoryId(event);
    String format = event.getFormat();
    log.debug("Deleting all search assets records for repository id: {}, format: {}", repositoryId, format);

    return searchTableStore.deleteAllSearchAssets(repositoryId, format);
  }
}
