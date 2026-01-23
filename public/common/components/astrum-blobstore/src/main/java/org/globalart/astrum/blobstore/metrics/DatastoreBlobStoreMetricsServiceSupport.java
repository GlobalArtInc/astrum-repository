
package org.globalart.astrum.blobstore.metrics;

import java.io.IOException;
import java.util.Collections;
import java.util.HashMap;
import java.util.Map;

import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.blobstore.api.OperationMetrics;
import org.globalart.astrum.blobstore.api.OperationType;
import org.globalart.astrum.blobstore.api.metrics.BlobStoreMetricsEntity;
import org.globalart.astrum.blobstore.api.metrics.BlobStoreMetricsService;
import org.globalart.astrum.blobstore.api.metrics.BlobStoreMetricsStore;
import org.globalart.astrum.blobstore.api.metrics.DatastoreBlobStoreMetricsContainer;
import org.globalart.astrum.common.scheduling.PeriodicJobService;
import org.globalart.astrum.common.scheduling.PeriodicJobService.PeriodicJob;
import org.globalart.astrum.common.stateguard.StateGuardLifecycleSupport;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Base class for {@link BlobStoreMetricsService} implementations.
 */
public abstract class DatastoreBlobStoreMetricsServiceSupport<B extends BlobStore>
    extends StateGuardLifecycleSupport
    implements BlobStoreMetricsService<B>
{
  private final int metricsFlushPeriodSeconds;

  private final PeriodicJobService jobService;

  private final DatastoreBlobStoreMetricsContainer datastoreBlobStoreMetricsContainer;

  private PeriodicJob metricsWritingJob;

  protected final BlobStoreMetricsStore blobStoreMetricsStore;

  protected B blobStore;

  protected DatastoreBlobStoreMetricsServiceSupport(
      final int metricsFlushPeriodSeconds,
      final PeriodicJobService jobService,
      final BlobStoreMetricsStore blobStoreMetricsStore)
  {
    this.metricsFlushPeriodSeconds = metricsFlushPeriodSeconds;
    this.jobService = checkNotNull(jobService);
    this.blobStoreMetricsStore = checkNotNull(blobStoreMetricsStore);

    this.datastoreBlobStoreMetricsContainer = new DatastoreBlobStoreMetricsContainer();
  }

  @Override
  protected void doStart() throws Exception {
    blobStoreMetricsStore.initializeMetrics(blobStore.getBlobStoreConfiguration().getName());
    jobService.startUsing();
    metricsWritingJob = jobService.schedule(() -> {
      if (datastoreBlobStoreMetricsContainer.metricsNeedFlushing()) {
        try {
          this.flush();
        }
        catch (Exception e) {
          log.error("Failed to save blobstore metrics to db", e);
        }
      }
    }, metricsFlushPeriodSeconds);
  }

  @Override
  public void doStop() throws Exception {
    metricsWritingJob.cancel();
    metricsWritingJob = null;
    jobService.stopUsing();
  }

  @Override
  public void init(final B blobStore) throws Exception {
    this.blobStore = blobStore;
    this.start();
  }

  @Override
  public void recordAddition(final long size) {
    datastoreBlobStoreMetricsContainer.recordAddition(size);
  }

  @Override
  public void recordDeletion(final long size) {
    datastoreBlobStoreMetricsContainer.recordDeletion(size);
  }

  @Override
  public Map<OperationType, OperationMetrics> getOperationMetrics() {
    BlobStoreMetricsEntity metricsEntity =
        blobStoreMetricsStore.get(this.blobStore.getBlobStoreConfiguration().getName());

    Map<OperationType, OperationMetrics> delta = getOperationMetricsDelta();

    OperationMetrics uploadMetrics = new OperationMetrics();
    uploadMetrics.setBlobSize(metricsEntity.getUploadBlobSize());
    uploadMetrics.setErrorRequests(metricsEntity.getUploadErrorRequests());
    uploadMetrics.setSuccessfulRequests(metricsEntity.getUploadSuccessfulRequests());
    uploadMetrics.setTimeOnRequests(metricsEntity.getUploadTimeOnRequests());
    uploadMetrics.add(delta.get(OperationType.UPLOAD));

    OperationMetrics downloadMetrics = new OperationMetrics();
    downloadMetrics.setBlobSize(metricsEntity.getDownloadBlobSize());
    downloadMetrics.setErrorRequests(metricsEntity.getDownloadErrorRequests());
    downloadMetrics.setSuccessfulRequests(metricsEntity.getDownloadSuccessfulRequests());
    downloadMetrics.setTimeOnRequests(metricsEntity.getDownloadTimeOnRequests());
    downloadMetrics.add(delta.get(OperationType.DOWNLOAD));

    Map<OperationType, OperationMetrics> operationMetricsMap = new HashMap<>();
    operationMetricsMap.put(OperationType.UPLOAD, uploadMetrics);
    operationMetricsMap.put(OperationType.DOWNLOAD, downloadMetrics);

    return Collections.unmodifiableMap(operationMetricsMap);
  }

  @Override
  public Map<OperationType, OperationMetrics> getOperationMetricsDelta() {
    return datastoreBlobStoreMetricsContainer.getOperationMetricsDelta();
  }

  @Override
  public void flush() throws IOException {
    OperationMetrics uploadMetrics =
        datastoreBlobStoreMetricsContainer.getOperationMetricsDelta().get(OperationType.UPLOAD);
    OperationMetrics downloadMetrics =
        datastoreBlobStoreMetricsContainer.getOperationMetricsDelta().get(OperationType.DOWNLOAD);

    BlobStoreMetricsEntity blobStoreMetricsEntity = new BlobStoreMetricsEntity()
        .setBlobStoreName(blobStore.getBlobStoreConfiguration().getName())
        .setBlobCount(datastoreBlobStoreMetricsContainer.blobCountDelta.getAndSet(0L))
        .setTotalSize(datastoreBlobStoreMetricsContainer.blobstoreUsageDelta.getAndSet(0L))
        .setDownloadBlobSize(downloadMetrics.getBlobSize())
        .setDownloadErrorRequests(downloadMetrics.getErrorRequests())
        .setDownloadSuccessfulRequests(downloadMetrics.getSuccessfulRequests())
        .setDownloadTimeOnRequests(downloadMetrics.getTimeOnRequests())
        .setUploadBlobSize(uploadMetrics.getBlobSize())
        .setUploadErrorRequests(uploadMetrics.getErrorRequests())
        .setUploadSuccessfulRequests(uploadMetrics.getSuccessfulRequests())
        .setDownloadTimeOnRequests(uploadMetrics.getTimeOnRequests());

    uploadMetrics.clear();
    downloadMetrics.clear();

    blobStoreMetricsStore.updateMetrics(blobStoreMetricsEntity);
  }

  @Override
  public void clearCountMetrics() {
    blobStoreMetricsStore.clearCountMetrics(blobStore.getBlobStoreConfiguration().getName());
  }

  @Override
  public void clearOperationMetrics() {
    datastoreBlobStoreMetricsContainer.getOperationMetricsDelta().values().forEach(OperationMetrics::clear);
    blobStoreMetricsStore.clearOperationMetrics(blobStore.getBlobStoreConfiguration().getName());
  }

  @Override
  public void remove() {
    blobStoreMetricsStore.remove(blobStore.getBlobStoreConfiguration().getName());
  }
}
