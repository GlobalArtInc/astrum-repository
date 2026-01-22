
package org.globalart.astrum.blobstore.api.metrics;

import java.util.Map;

import org.globalart.astrum.blobstore.api.OperationMetrics;
import org.globalart.astrum.blobstore.api.OperationType;

import org.junit.Before;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.hasKey;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;
import static org.hamcrest.Matchers.sameInstance;

/**
 * Tests for {@link DatastoreBlobStoreMetricsContainer}.
 */
public class DatastoreBlobStoreMetricsContainerTest
{
  private DatastoreBlobStoreMetricsContainer container;

  @Before
  public void setUp() {
    container = new DatastoreBlobStoreMetricsContainer();
  }

  @Test
  public void testDefaultState() {
    assertThat(container.blobstoreUsageDelta.get(), is(0L));
    assertThat(container.blobCountDelta.get(), is(0L));

    Map<OperationType, OperationMetrics> delta = container.getOperationMetricsDelta();
    assertThat(delta.size(), is(2));
    assertThat(delta, hasKey(OperationType.UPLOAD));
    assertThat(delta, hasKey(OperationType.DOWNLOAD));
    OperationMetrics uploadMetrics = delta.get(OperationType.UPLOAD);
    assertThat(uploadMetrics, notNullValue());
    assertThat(uploadMetrics.getBlobSize(), is(0L));
    assertThat(uploadMetrics.getSuccessfulRequests(), is(0L));
    assertThat(uploadMetrics.getErrorRequests(), is(0L));
    assertThat(uploadMetrics.getTimeOnRequests(), is(0L));

    OperationMetrics downloadMetrics = delta.get(OperationType.DOWNLOAD);
    assertThat(downloadMetrics, notNullValue());
    assertThat(downloadMetrics.getBlobSize(), is(0L));
    assertThat(downloadMetrics.getSuccessfulRequests(), is(0L));
    assertThat(downloadMetrics.getErrorRequests(), is(0L));
    assertThat(downloadMetrics.getTimeOnRequests(), is(0L));
  }

  @Test
  public void testGetOperationMetricsDeltaReturnsSameInstance() {
    assertThat(container.getOperationMetricsDelta(), sameInstance(container.operationMetricsDelta));
  }

  @Test
  public void testMetricsDoNotNeedFlushingWhenEmpty() {
    assertThat(container.metricsNeedFlushing(), is(false));
  }

  @Test
  public void testMetricsNeedFlushingWhenBlobCountDeltaIsNonZero() {
    container.blobCountDelta.set(5L);

    assertThat(container.metricsNeedFlushing(), is(true));
  }

  @Test
  public void testMetricsNeedFlushingWhenBlobCountDeltaIsNegative() {
    // the operand uses != 0 (not > 0), so a negative count delta must also trigger flushing;
    // usage stays 0 to isolate the first OR operand
    container.blobCountDelta.set(-3L);

    assertThat(container.blobstoreUsageDelta.get(), is(0L));
    assertThat(container.metricsNeedFlushing(), is(true));
  }

  @Test
  public void testMetricsNeedFlushingWhenBlobstoreUsageDeltaIsNonZero() {
    // count is back to zero but usage is not, so the second operand of the OR must trigger flushing
    container.recordAddition(100L);
    container.recordDeletion(30L);

    assertThat(container.blobCountDelta.get(), is(0L));
    assertThat(container.blobstoreUsageDelta.get(), is(70L));
    assertThat(container.metricsNeedFlushing(), is(true));
  }

  @Test
  public void testMetricsNeedFlushingWhenBlobstoreUsageDeltaIsNegative() {
    // the operand uses != 0 (not > 0), so a negative usage delta must also trigger flushing;
    // count stays 0 to isolate the second OR operand
    container.blobstoreUsageDelta.set(-50L);

    assertThat(container.blobCountDelta.get(), is(0L));
    assertThat(container.metricsNeedFlushing(), is(true));
  }

  @Test
  public void testMetricsNeedFlushingWhenBlobSizeIsPositive() {
    container.getOperationMetricsDelta().get(OperationType.UPLOAD).setBlobSize(1L);

    assertThat(container.metricsNeedFlushing(), is(true));
  }

  @Test
  public void testMetricsNeedFlushingWhenErrorRequestsArePositive() {
    container.getOperationMetricsDelta().get(OperationType.DOWNLOAD).setErrorRequests(1L);

    assertThat(container.metricsNeedFlushing(), is(true));
  }

  @Test
  public void testMetricsNeedFlushingWhenSuccessfulRequestsArePositive() {
    container.getOperationMetricsDelta().get(OperationType.UPLOAD).setSuccessfulRequests(1L);

    assertThat(container.metricsNeedFlushing(), is(true));
  }

  @Test
  public void testMetricsNeedFlushingWhenTimeOnRequestsIsPositive() {
    container.getOperationMetricsDelta().get(OperationType.DOWNLOAD).setTimeOnRequests(1L);

    assertThat(container.metricsNeedFlushing(), is(true));
  }

  @Test
  public void testRecordAddition() {
    container.recordAddition(100L);

    assertThat(container.blobstoreUsageDelta.get(), is(100L));
    assertThat(container.blobCountDelta.get(), is(1L));
  }

  @Test
  public void testRecordAdditionAccumulates() {
    container.recordAddition(100L);
    container.recordAddition(50L);

    assertThat(container.blobstoreUsageDelta.get(), is(150L));
    assertThat(container.blobCountDelta.get(), is(2L));
  }

  @Test
  public void testRecordDeletion() {
    container.recordDeletion(40L);

    assertThat(container.blobstoreUsageDelta.get(), is(-40L));
    assertThat(container.blobCountDelta.get(), is(-1L));
  }

  @Test
  public void testRecordDeletionAccumulates() {
    container.recordDeletion(40L);
    container.recordDeletion(10L);

    assertThat(container.blobstoreUsageDelta.get(), is(-50L));
    assertThat(container.blobCountDelta.get(), is(-2L));
  }

  @Test
  public void testRecordAdditionAndDeletionNetOut() {
    container.recordAddition(100L);
    container.recordDeletion(100L);

    assertThat(container.blobstoreUsageDelta.get(), is(0L));
    assertThat(container.blobCountDelta.get(), is(0L));
  }

  @Test(expected = IllegalArgumentException.class)
  public void testRecordDeletionRejectsNegativeSize() {
    container.recordDeletion(-30L);
  }

  @Test(expected = IllegalArgumentException.class)
  public void testRecordAdditionRejectsNegativeSize() {
    container.recordAddition(-30L);
  }
}
