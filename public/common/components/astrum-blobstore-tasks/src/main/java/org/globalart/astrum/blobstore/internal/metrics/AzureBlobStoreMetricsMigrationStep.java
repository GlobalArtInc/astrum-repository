
package org.globalart.astrum.blobstore.internal.metrics;

import org.springframework.stereotype.Component;

/**
 * Migration step to move metrics from properties files in the blob store to the DB
 */
@Component
public class AzureBlobStoreMetricsMigrationStep
    extends BlobStoreMetricsDatabaseMigrationStepSupport
{

  private static final String BLOB_STORE_TYPE = "Azure Cloud Storage";

  public AzureBlobStoreMetricsMigrationStep() {
    super(BLOB_STORE_TYPE);
  }

  @Override
  public Integer getChecksum() {
    return 0;
  }
}
