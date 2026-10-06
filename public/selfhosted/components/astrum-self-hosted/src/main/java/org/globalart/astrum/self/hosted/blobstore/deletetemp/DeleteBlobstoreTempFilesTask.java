
package org.globalart.astrum.self.hosted.blobstore.deletetemp;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.blobstore.api.BlobStoreManager;
import org.globalart.astrum.scheduling.Cancelable;
import org.globalart.astrum.scheduling.TaskSupport;

import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.self.hosted.blobstore.deletetemp.DeleteBlobstoreTempFilesTaskDescriptor.BLOB_STORE_NAME_FIELD_ID;
import static org.globalart.astrum.self.hosted.blobstore.deletetemp.DeleteBlobstoreTempFilesTaskDescriptor.DAYS_OLDER_THAN;

@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class DeleteBlobstoreTempFilesTask
    extends TaskSupport
    implements Cancelable
{
  private final BlobStoreManager blobStoreManager;

  @Autowired
  public DeleteBlobstoreTempFilesTask(final BlobStoreManager blobStoreManager) {
    this.blobStoreManager = checkNotNull(blobStoreManager);
  }

  @Override
  protected Object execute() throws Exception {
    BlobStore blobStore = blobStoreManager.get(getBlobStoreField());
    if (blobStore != null) {
      int daysOlderThan = getDaysOlderThan() == null ? 0 : Integer.parseInt(getDaysOlderThan());
      blobStore.deleteTempFiles(daysOlderThan);
    }
    else {
      log.warn("Unable to find blob store: {}", getBlobStoreField());
    }
    return null;
  }

  @Override
  public String getMessage() {
    return "Deleting " + getBlobStoreField() + " blob store temporary files";
  }

  private String getBlobStoreField() {
    return getConfiguration().getString(BLOB_STORE_NAME_FIELD_ID);
  }

  private String getDaysOlderThan() {
    return getConfiguration().getString(DAYS_OLDER_THAN);
  }
}
