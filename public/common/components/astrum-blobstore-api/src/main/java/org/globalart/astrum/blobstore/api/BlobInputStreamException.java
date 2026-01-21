
package org.globalart.astrum.blobstore.api;

import javax.annotation.Nullable;

/**
 * Exception used to indicate when an existing Blob's Input Stream cannot be opened.
 *
 * @since 3.31
 */
public class BlobInputStreamException
    extends BlobStoreException
{
  public BlobInputStreamException(final Throwable cause, @Nullable final BlobId blobId) {
    super(cause, blobId);
  }
}
