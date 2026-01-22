
package org.globalart.astrum.blobstore.api.metrics;

import java.util.EnumMap;
import java.util.Map;
import java.util.concurrent.atomic.AtomicLong;

import org.globalart.astrum.blobstore.api.OperationMetrics;
import org.globalart.astrum.blobstore.api.OperationType;

import static com.google.common.base.Preconditions.checkArgument;

/**
 * Class to hold and manage the metrics for a blobstore.
 */
public class DatastoreBlobStoreMetricsContainer
{
  public final AtomicLong blobstoreUsageDelta = new AtomicLong(0L);

  public final AtomicLong blobCountDelta = new AtomicLong(0L);

  public final Map<OperationType, OperationMetrics> operationMetricsDelta = new EnumMap<>(OperationType.class);

  public DatastoreBlobStoreMetricsContainer() {
    operationMetricsDelta.put(OperationType.UPLOAD, new OperationMetrics());
    operationMetricsDelta.put(OperationType.DOWNLOAD, new OperationMetrics());
  }

  public boolean metricsNeedFlushing() {
    if (blobCountDelta.get() != 0 || blobstoreUsageDelta.get() != 0) {
      return true;
    }

    return operationMetricsDelta
        .values()
        .stream()
        .anyMatch(
            operationMetrics -> operationMetrics.getBlobSize() > 0 ||
                operationMetrics.getErrorRequests() > 0 ||
                operationMetrics.getSuccessfulRequests() > 0 ||
                operationMetrics.getTimeOnRequests() > 0);
  }

  public Map<OperationType, OperationMetrics> getOperationMetricsDelta() {
    return operationMetricsDelta;
  }

  public void recordAddition(final long size) {
    checkArgument(size >= 0, "size must be non-negative");
    blobstoreUsageDelta.getAndAdd(size);
    blobCountDelta.incrementAndGet();
  }

  public void recordDeletion(final long size) {
    checkArgument(size >= 0, "size must be non-negative");
    blobstoreUsageDelta.getAndAdd(-size);
    blobCountDelta.decrementAndGet();
  }
}
