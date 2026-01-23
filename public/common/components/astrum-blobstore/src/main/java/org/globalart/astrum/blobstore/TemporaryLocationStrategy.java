
package org.globalart.astrum.blobstore;

import org.globalart.astrum.blobstore.api.BlobId;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Stores temporary blobs in a single directory separate from the blobs holding actual saved content.
 *
 * @since 3.1
 */
public class TemporaryLocationStrategy
    extends LocationStrategySupport
{
  @Override
  public String location(final BlobId blobId) {
    checkNotNull(blobId);
    return String.format("tmp/%s", escapeFilename(blobId.asUniqueString()));
  }
}
