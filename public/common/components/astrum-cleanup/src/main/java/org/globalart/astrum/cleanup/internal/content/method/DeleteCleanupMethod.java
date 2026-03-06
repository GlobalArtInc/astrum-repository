
package org.globalart.astrum.cleanup.internal.content.method;

import java.util.List;
import java.util.function.BooleanSupplier;
import java.util.stream.Stream;
import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.cleanup.internal.method.CleanupMethod;
import org.globalart.astrum.common.db.DatabaseCheck;
import org.globalart.astrum.common.entity.Continuations;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.browse.BrowseFacet;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.content.maintenance.ContentMaintenanceFacet;
import org.globalart.astrum.repository.content.store.InternalIds;
import org.globalart.astrum.repository.task.DeletionProgress;
import org.globalart.astrum.scheduling.TaskInterruptedException;

import com.google.common.collect.Iterators;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * Provides a delete mechanism for cleanup
 *
 * @since 3.29
 */
@Component
public class DeleteCleanupMethod
    implements CleanupMethod
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final DatabaseCheck databaseCheck;

  @Autowired
  public DeleteCleanupMethod(final DatabaseCheck databaseCheck) {
    this.databaseCheck = checkNotNull(databaseCheck);
  }

  @Override
  public DeletionProgress run(
      final Repository repository,
      final Stream<FluentComponent> components,
      final BooleanSupplier cancelledCheck)
  {
    ContentMaintenanceFacet maintenance = repository.facet(ContentMaintenanceFacet.class);
    DeletionProgress progress = new DeletionProgress();

    Iterators.partition(components.iterator(), Continuations.BROWSE_LIMIT)
        .forEachRemaining((batch) -> deleteBatch(maintenance, batch.stream(), progress, cancelledCheck));

    return progress;
  }

  private void deleteBatch(
      final ContentMaintenanceFacet maintenance,
      final Stream<FluentComponent> batch,
      final DeletionProgress progress,
      final BooleanSupplier cancelledCheck)
  {

    if (cancelledCheck.getAsBoolean()) {
      throw new TaskInterruptedException(
          String.format("Thread '%s' is canceled", Thread.currentThread().getName()),
          true);
    }

    // Collect the stream into a list to allow reuse
    List<FluentComponent> components = batch.toList();

    if (isPostgresql()) {
      deleteBrowseNodes(components);
    }

    logAssetsBeingDeleted(components);

    progress.addComponentCount(maintenance.deleteComponents(components.stream()));
  }

  private void logAssetsBeingDeleted(final List<FluentComponent> components) {
    if (log.isDebugEnabled()) {
      components.forEach(component -> component.assets().forEach(asset -> {
        String repositoryName = asset.repository().getName();
        String assetPath = asset.path();

        asset.blob().ifPresent(blob -> {
          log.debug("Deleting asset - repository: {}, path: {}, blobRef: {}, size: {} bytes",
              repositoryName, assetPath, blob.blobRef(), blob.blobSize());
        });
      }));
    }
  }

  private void deleteBrowseNodes(final List<FluentComponent> components) {
    components.forEach(component -> component.assets().forEach(asset -> {
      Integer internalAssetId = InternalIds.internalAssetId(asset);
      asset.repository()
          .optionalFacet(BrowseFacet.class)
          .ifPresent(facet -> facet.deleteByAssetIdAndPath(internalAssetId, asset.path()));
    }));
  }

  private boolean isPostgresql() {
    return this.databaseCheck.isPostgresql();
  }
}
