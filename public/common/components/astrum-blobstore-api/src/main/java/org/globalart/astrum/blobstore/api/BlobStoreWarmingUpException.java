
package org.globalart.astrum.blobstore.api;

/**
 * Exception thrown when a blob store is still warming up connection pools during startup.
 *
 * This exception indicates a temporary, retry-able condition where the blob store's
 * connection pool (S3/Azure) is still being initialized. Clients should retry the request
 * after a brief delay (typically 1-2 seconds).
 *
 * This is distinct from {@link org.globalart.astrum.common.stateguard.InvalidStateException}
 * which indicates permanent invalid states, and from {@link org.globalart.astrum.repository.MissingBlobException}
 * which indicates data corruption.
 */
public class BlobStoreWarmingUpException
    extends RuntimeException
{
  private final String blobStoreName;

  public BlobStoreWarmingUpException(final String blobStoreName) {
    super("Blob store '" + blobStoreName + "' is still warming up connection pools, please retry in a moment");
    this.blobStoreName = blobStoreName;
  }

  public String getBlobStoreName() {
    return blobStoreName;
  }
}
