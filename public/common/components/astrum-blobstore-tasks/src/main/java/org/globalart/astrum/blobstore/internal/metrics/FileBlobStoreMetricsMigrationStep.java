
package org.globalart.astrum.blobstore.internal.metrics;

import org.globalart.astrum.blobstore.file.FileBlobStore;
import org.springframework.stereotype.Component;

/**
 * Migration step to move metrics from properties files in the blob store to the DB
 */
@Component
public class FileBlobStoreMetricsMigrationStep
    extends BlobStoreMetricsDatabaseMigrationStepSupport
{

  public FileBlobStoreMetricsMigrationStep() {
    super(FileBlobStore.TYPE);
  }

  @Override
  public Integer getChecksum() {
    return 0;
  }
}
