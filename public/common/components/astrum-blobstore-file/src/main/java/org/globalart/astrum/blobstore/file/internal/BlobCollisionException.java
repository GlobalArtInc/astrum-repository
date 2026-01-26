
package org.globalart.astrum.blobstore.file.internal;

import org.globalart.astrum.blobstore.api.BlobId;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Thrown when a blob already exists for the generated {@link BlobId}.
 *
 * @since 3.4
 */
public class BlobCollisionException
    extends RuntimeException
{
  private final BlobId blobId;

  public BlobCollisionException(final BlobId blobId) {
    this.blobId = checkNotNull(blobId);
  }

  /**
   * The BlobId of the blob related to this exception.
   */
  public BlobId getBlobId() {
    return blobId;
  }
}
