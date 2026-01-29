
package org.globalart.astrum.repository;

import org.globalart.astrum.blobstore.api.BlobRef;

/**
 * Thrown when attempting to access blob content which is now missing from the blobstore.
 *
 * @since 3.22
 */
public class MissingBlobException
    extends IllegalStateException
{
  private final BlobRef blobRef;

  public MissingBlobException(final BlobRef blobRef) {
    super(String.format("Blob %s exists in metadata, but is missing from the blobstore", blobRef));
    this.blobRef = blobRef;
  }

  public BlobRef getBlobRef() {
    return blobRef;
  }
}
