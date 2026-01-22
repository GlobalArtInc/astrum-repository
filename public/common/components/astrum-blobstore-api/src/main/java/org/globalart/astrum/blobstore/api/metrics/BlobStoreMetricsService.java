
package org.globalart.astrum.blobstore.api.metrics;

import java.io.IOException;
import java.util.Map;

import org.globalart.astrum.common.lifecycle.Lifecycle;
import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.blobstore.api.BlobStoreMetrics;
import org.globalart.astrum.blobstore.api.OperationMetrics;
import org.globalart.astrum.blobstore.api.OperationType;

/**
 * Service for handling metrics for blobstores. They are NOT singletons, there is an instance for each blobstore.
 */
public interface BlobStoreMetricsService<B extends BlobStore>
    extends Lifecycle
{
  /**
   * Retrieve the metrics for the attached blobstore.
   */
  BlobStoreMetrics getMetrics();

  void init(B blobStore) throws Exception;

  /**
   * Record a new blob being added to the blob store.
   *
   * @param size Size of the newly added blob in bytes; must be non-negative.
   */
  void recordAddition(final long size);

  /**
   * Record a blob being deleted from the from blobstore.
   *
   * @param size Size of the deleted blob in bytes; must be non-negative.
   */
  void recordDeletion(final long size);

  /**
   * Get the operation metrics for the attached blobstore.
   */
  Map<OperationType, OperationMetrics> getOperationMetrics();

  /**
   * Get the current delta set of operation metrics. This used to track operation metric data to be flushed.
   */
  Map<OperationType, OperationMetrics> getOperationMetricsDelta();

  /**
   * Reset the total metrics (blob count / size)
   */
  void clearCountMetrics();

  /**
   * Reset the operational metrics.
   */
  void clearOperationMetrics();

  /**
   * Flush the metrics to the metric store.
   */
  void flush() throws IOException;

  /**
   * Remove metrics for the given blobstore.
   */
  void remove();
}
