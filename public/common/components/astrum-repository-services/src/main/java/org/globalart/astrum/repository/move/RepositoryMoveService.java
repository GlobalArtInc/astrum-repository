
package org.globalart.astrum.repository.move;

import javax.annotation.Nullable;

import org.globalart.astrum.blobstore.api.Blob;
import org.globalart.astrum.blobstore.api.BlobRef;

/**
 *
 * @since 3.28
 */
public interface RepositoryMoveService
{
  /**
   *
   * @param blobRef
   * @return
   */
  @Nullable
  Blob getIfBeingMoved(BlobRef blobRef, String sourceRepositoryName);
}
