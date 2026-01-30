
package org.globalart.astrum.repository.tools;

import java.util.function.Consumer;

import org.globalart.astrum.blobstore.api.BlobId;
import org.globalart.astrum.repository.Repository;

/**
 * Detects orphaned blobs (i.e. non-deleted blobs that exist in the blobstore but not the asset table)
 *
 * @since 3.13
 */
public interface OrphanedBlobFinder
{
  /**
   * Delete orphaned blobs for all repositories
   */
  void delete();

  /**
   * Delete orphaned blobs associated with a given repository
   *
   * @param repository - where to look for orphaned blobs
   */
  void delete(final Repository repository);

  /**
   * Look for orphaned blobs in a given repository and callback for each blobId found
   *
   * @param repository - where to look for orphaned blobs
   * @param handler - callback to handle an orphaned blob
   */
  void detect(final Repository repository, final Consumer<BlobId> handler);

}
