
package org.globalart.astrum.blobstore.api.metrics;

/**
 * Store for managing DB based blobstore metrics.
 */
public interface BlobStoreMetricsStore
{
  /**
   * Update metrics for the given {@link BlobStoreMetricsEntity}.
   */
  void updateMetrics(BlobStoreMetricsEntity blobStoreMetricsEntity);

  /**
   * Retrieve metrics for the given blobstore name.
   */
  BlobStoreMetricsEntity get(String blobStoreName);

  /**
   * Remove metrics for the given blobstore name.
   */
  void remove(String blobStoreName);

  /**
   * Clear the operation metrics for the given blobstore name.
   */
  void clearOperationMetrics(String blobStoreName);

  /**
   * Clear the summary metrics for the given blob store name.
   * (total size , blob count)
   */
  void clearCountMetrics(String blobStoreName);

  /**
   * Initialize the metrics for the given blobstore.
   */
  void initializeMetrics(String blobStoreName);

  /**
   * Rename the metrics record for a blob store.
   */
  void rename(String oldName, String newName);
}
