
package org.globalart.astrum.repository.search.sql.index;

import java.util.Collection;

import org.globalart.astrum.common.entity.Continuation;
import org.globalart.astrum.common.entity.EntityId;
import org.globalart.astrum.common.stateguard.Guarded;
import org.globalart.astrum.logging.task.ProgressLogIntervalHelper;
import org.globalart.astrum.repository.FacetSupport;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.content.fluent.FluentComponents;
import org.globalart.astrum.repository.content.search.SearchFacet;
import org.globalart.astrum.repository.search.sql.store.SearchStore;

import com.google.common.base.Stopwatch;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Primary;
import org.springframework.context.annotation.Scope;
import org.springframework.core.Ordered;
import org.springframework.core.annotation.Order;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.base.Preconditions.checkState;
import static org.apache.shiro.util.CollectionUtils.isEmpty;
import static org.globalart.astrum.repository.FacetSupport.State.STARTED;
import static org.globalart.astrum.scheduling.CancelableHelper.checkCancellation;

/**
 * The {@link SearchFacet} implementation for the SQL Table search.
 */
@Primary
@Component
@Order(Ordered.HIGHEST_PRECEDENCE)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class SqlSearchFacetImpl
    extends FacetSupport
    implements SearchFacet
{
  private final SearchStore store;

  private final SqlSearchIndexService sqlSearchIndexService;

  private final int batchSize;

  @Autowired
  public SqlSearchFacetImpl(
      final SearchStore store,
      final SqlSearchIndexService sqlSearchIndexService,
      @Value("${astrum.rebuild.search.batchSize:500}") final int batchSize)
  {
    this.store = checkNotNull(store);
    this.sqlSearchIndexService = checkNotNull(sqlSearchIndexService);

    checkState(batchSize >= 1, "batchSize should be greater than 1");
    this.batchSize = batchSize;
  }

  @Guarded(by = STARTED)
  @Override
  public void rebuildIndex() {
    Repository repository = getRepository();
    String repositoryFormat = repository.getFormat().getValue();
    String repositoryName = repository.getName();
    log.info("Starting regenerate the search data for the {} repository: {}", repositoryFormat, repositoryName);

    process(repository);

    log.info("Finish regenerating the search data for the {} repository: {}", repositoryFormat, repositoryName);
  }

  private void process(final Repository repository) {
    String repositoryFormat = repository.getFormat().getValue();
    Integer repositoryId = repository.facet(ContentFacet.class).contentRepositoryId();
    log.info("Processing the {} repository: {}", repositoryFormat, repository.getName());

    populateComponents(repository);

    // Remove orphaned records: search_components entries whose component_id no longer
    // exists in the format-specific component table. Using NOT EXISTS is correct
    // regardless of blob age — last_modified reflects blob.blobCreated() which can be
    // arbitrarily old and must not be used for orphan detection.
    log.info("Removing orphaned search records for repository: {}", repository.getName());
    // deleteOrphanedComponents must run before deleteOrphanedAssets because
    // deleteOrphanedAssets relies on search_components rows to identify which
    // asset rows are still valid. Reversing the order would cause all assets
    // to appear orphaned and be incorrectly deleted.
    store.deleteOrphanedComponents(repositoryId, repositoryFormat);
    store.deleteOrphanedAssets(repositoryId, repositoryFormat);
  }

  private void populateComponents(final Repository repository) {
    final FluentComponents fluentComponents = facet(ContentFacet.class).components();
    long processed = 0L;
    int totalComponents = fluentComponents.count();
    try (ProgressLogIntervalHelper progressLogger = new ProgressLogIntervalHelper(log, 60)) {
      Stopwatch sw = Stopwatch.createStarted();
      Continuation<FluentComponent> components = fluentComponents.browse(batchSize, null);
      while (!isEmpty(components)) {
        checkCancellation();
        sqlSearchIndexService.indexBatch(components, repository);
        processed += components.size();
        progressLogger
            .info("Indexed {} / {} {} components in {}", processed, totalComponents, repository.getName(), sw);

        components = fluentComponents.browse(batchSize, components.nextContinuationToken());
      }
      progressLogger.flush(); // ensure the final progress message is flushed
    }
  }

  @Guarded(by = STARTED)
  @Override
  public void index(final Collection<EntityId> componentIds) {
    log.debug("Indexing... {}", componentIds);
    sqlSearchIndexService.index(componentIds, getRepository());
  }

  @Guarded(by = STARTED)
  @Override
  public void purge(final Collection<EntityId> componentIds) {
    log.debug("Purging... {}", componentIds);
    sqlSearchIndexService.purge(componentIds, getRepository());
  }

  /**
   * Deletes all search records for the repository.
   */
  @Override
  protected void doDelete() {
    Repository repository = getRepository();
    String repositoryFormat = repository.getFormat().getValue();
    String repositoryName = repository.getName();
    Integer repositoryId = repository.facet(ContentFacet.class).contentRepositoryId();

    log.debug("deleting search records for repository: {} ({})", repositoryName, repositoryFormat);

    store.deleteAllForRepository(repositoryId, repositoryFormat);
    store.deleteAllSearchAssets(repositoryId, repositoryFormat);
  }
}
