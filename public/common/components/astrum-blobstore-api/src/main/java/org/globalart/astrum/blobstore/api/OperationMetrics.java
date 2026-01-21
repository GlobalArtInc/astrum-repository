
package org.globalart.astrum.blobstore.api;

import java.util.concurrent.atomic.AtomicLong;

import org.globalart.astrum.common.math.Math2;

/**
 * Blob store metrics to monitor method invocation by type, see {@code OperationType}.
 *
 * @since 3.38
 */
public class OperationMetrics
{
  // in bytes
  private final AtomicLong blobSize = new AtomicLong();

  private final AtomicLong successfulRequests = new AtomicLong();

  // in millis
  private final AtomicLong timeOnRequests = new AtomicLong();

  private final AtomicLong errorRequests = new AtomicLong();

  public long getBlobSize() {
    return blobSize.get();
  }

  public long getSuccessfulRequests() {
    return successfulRequests.get();
  }

  public long getTimeOnRequests() {
    return timeOnRequests.get();
  }

  public long getErrorRequests() {
    return errorRequests.get();
  }

  public void setBlobSize(final long blobSize) {
    this.blobSize.set(blobSize);
  }

  public void setSuccessfulRequests(final long successfulRequests) {
    this.successfulRequests.set(successfulRequests);
  }

  public void setTimeOnRequests(final long timeOnRequests) {
    this.timeOnRequests.set(timeOnRequests);
  }

  public void setErrorRequests(final long errorRequests) {
    this.errorRequests.set(errorRequests);
  }

  public void addBlobSize(final long blobSize) {
    this.blobSize.updateAndGet(size -> size + blobSize);
  }

  public void addTimeOnRequests(final long timeOnRequests) {
    this.timeOnRequests.updateAndGet(time -> time + timeOnRequests);
  }

  public void addSuccessfulRequest() {
    this.successfulRequests.incrementAndGet();
  }

  public void addErrorRequest() {
    this.errorRequests.incrementAndGet();
  }

  /**
   * Clear all metrics.
   */
  public void clear() {
    blobSize.set(0L);
    successfulRequests.set(0L);
    timeOnRequests.set(0L);
    errorRequests.set(0L);
  }

  /**
   * Aggregate the existing metrics with the provided ones.
   *
   * @param operationMetrics metrics to add.
   * @return the aggregated metrics.
   */
  public OperationMetrics add(final OperationMetrics operationMetrics) {
    OperationMetrics metrics = new OperationMetrics();
    metrics.setBlobSize(Math2.addClamped(blobSize.get(), operationMetrics.getBlobSize()));
    metrics.setSuccessfulRequests(Math2.addClamped(successfulRequests.get(), operationMetrics.getSuccessfulRequests()));
    metrics.setTimeOnRequests(Math2.addClamped(timeOnRequests.get(), operationMetrics.getTimeOnRequests()));
    metrics.setErrorRequests(Math2.addClamped(errorRequests.get(), operationMetrics.getErrorRequests()));

    return metrics;
  }

  public String toString() {
    return "OperationMetrics{" +
        "blobSize=" + blobSize +
        ", successfulRequests=" + successfulRequests +
        ", timeOnRequests=" + timeOnRequests +
        ", errorRequests=" + errorRequests +
        '}';
  }
}
