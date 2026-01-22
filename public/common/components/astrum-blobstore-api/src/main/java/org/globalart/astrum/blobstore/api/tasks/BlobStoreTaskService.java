
package org.globalart.astrum.blobstore.api.tasks;

public interface BlobStoreTaskService
{
  boolean isAnyTaskInUseForBlobStore(final String blobStoreName);

  int countTasksInUseForBlobStore(final String blobStoreName);
}
