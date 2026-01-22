
package org.globalart.astrum.blobstore.api.metrics;

import java.util.Map;

import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.blobstore.api.BlobStoreMetrics;
import org.globalart.astrum.blobstore.api.OperationMetrics;
import org.globalart.astrum.blobstore.api.OperationType;

/**
 * Read blob store metrics from properties files
 */
public interface BlobStoreMetricsPropertiesReader<B extends BlobStore>
{
  /**
   * Retrieve the metrics for the attached blobstore.
   */
  BlobStoreMetrics getMetrics();

  /**
   * Get the operation metrics for the attached blobstore.
   */
  Map<OperationType, OperationMetrics> getOperationMetrics();

  void init(B blobStore) throws Exception;
}
