
package org.globalart.astrum.blobstore;

import org.globalart.astrum.blobstore.api.BlobId;

import static com.google.common.base.Preconditions.checkArgument;
import static com.google.common.base.Preconditions.checkNotNull;
import static java.lang.String.format;

/**
 * {@link LocationStrategy} for resolving locations to
 * {@link org.globalart.astrum.blobstore.api.BlobStore#DIRECT_PATH_BLOB_HEADER} blobs.
 *
 * @since 3.8
 */
public class DirectPathLocationStrategy
    extends LocationStrategySupport
{
  public static final String DIRECT_PATH_ROOT = "directpath";

  public static final String DIRECT_PATH_PREFIX = "path$";

  @Override
  public String location(final BlobId blobId) {
    checkNotNull(blobId);
    String realBlobIdPath = blobId.asUniqueString().replace(DIRECT_PATH_PREFIX, "");
    checkArgument(!realBlobIdPath.contains(".."), "Traversal not allowed with direct blobs");
    return format("%s/%s", DIRECT_PATH_ROOT, realBlobIdPath);
  }
}
