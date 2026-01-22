
package org.globalart.astrum.blobstore.api.tasks;

public interface BlobStoreConsumer
{
  boolean isBlobStoreInUse(final String blobStoreName);

  int blobStoreUsageCount(final String blobStoreName);
}
