
package org.globalart.astrum.coreui.internal.blobstore;

public class BlobStoreUsageUIResponse
{
  private final long repositoryUsage;

  private final long blobStoreUsage;

  public BlobStoreUsageUIResponse(final long repositoryUsage, final long blobStoreUsage) {
    this.repositoryUsage = repositoryUsage;
    this.blobStoreUsage = blobStoreUsage;
  }

  public long getRepositoryUsage() {
    return repositoryUsage;
  }

  public long getBlobStoreUsage() {
    return blobStoreUsage;
  }
}
