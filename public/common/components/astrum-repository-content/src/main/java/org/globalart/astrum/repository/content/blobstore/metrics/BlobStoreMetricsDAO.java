
package org.globalart.astrum.repository.content.blobstore.metrics;

import org.globalart.astrum.blobstore.api.metrics.BlobStoreMetricsEntity;
import org.globalart.astrum.datastore.api.ContentDataAccess;
import org.apache.ibatis.annotations.Param;

/**
 * DAO for access blobstore metrics
 */
public interface BlobStoreMetricsDAO
    extends ContentDataAccess
{
  /**
   * Update metrics in the db for the given {@link BlobStoreMetricsEntity}
   */
  void updateMetrics(BlobStoreMetricsEntity blobStoreMetricsEntity);

  /**
   * Retrieve {@link BlobStoreMetricsEntity} blobstore metrics for the given blobstore name.
   */
  BlobStoreMetricsEntity get(String blobStoreName);

  /**
   * Remove metrics for the given blobstore name.
   */
  void remove(String blobStoreName);

  /**
   * Initialize the metrics for the given blobstore.
   */
  void initializeMetrics(String blobStoreName);

  /**
   * Clears the operational metrics associated with the blob store
   */
  void clearOperationMetrics(String blobStoreName);

  /**
   * Clears the summary metrics associated to the given blob store (total size , blob count).
   */
  void clearCountMetrics(String blobStoreName);

  /**
   * Rename the metrics record for a blob store.
   */
  void rename(@Param("oldName") String oldName, @Param("newName") String newName);
}
