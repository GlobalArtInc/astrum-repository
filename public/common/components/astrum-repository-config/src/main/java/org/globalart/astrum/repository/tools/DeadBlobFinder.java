
package org.globalart.astrum.repository.tools;

import java.util.List;
import java.util.function.Consumer;
import jakarta.validation.constraints.NotNull;

import org.globalart.astrum.repository.Repository;

/**
 * Examines Asset metadata and confirms the sha1 of all referenced blobs. Reports on any instances where
 * Blob binary is missing or indicates a different sha1 than that stored in the DB.
 * 
 * @since 3.3
 */
public interface DeadBlobFinder<A>
{
  /**
   * Based on the db metadata, confirm that all Blobs exist and sha1 values match. Can optionally ignore any records
   * that don't have a blobRef, which is expected for NuGet search results.
   * 
   * @parem repository The Repository to inspect
   */
  default List<DeadBlobResult<A>> find(@NotNull final Repository repository) {
    return find(repository, true);
  }

  /**
   * Based on the db metadata, confirm that all Blobs exist and sha1 values match. Can optionally ignore any records
   * that don't have a blobRef, which is expected for NuGet search results.
   * 
   * @parem repository The Repository to inspect
   * @param ignoreMissingBlobRefs
   */
  List<DeadBlobResult<A>> find(@NotNull final Repository repository, boolean ignoreMissingBlobRefs);

  /**
   * Based on the db metadata, confirm that all Blobs exist and sha1 values match. Can optionally ignore any records
   * that don't have a blobRef, which is expected for NuGet search results. It finds and processes the assets in
   * batches.
   * 
   * @parem repository The Repository to inspect
   * @param ignoreMissingBlobRefs
   * @param batchSize
   * @param resultProcessor to process the results
   */
  public void findAndProcessBatch(
      @NotNull final Repository repository,
      final boolean ignoreMissingBlobRefs,
      final int batchSize,
      final Consumer<DeadBlobResult<A>> resultProcessor);

}
