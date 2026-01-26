
package org.globalart.astrum.blobstore.file.internal.datastore.metrics;

import java.io.IOException;
import java.io.UncheckedIOException;
import java.nio.file.FileStore;
import java.nio.file.Files;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.blobstore.AccumulatingBlobStoreMetrics;
import org.globalart.astrum.blobstore.api.BlobStoreMetrics;
import org.globalart.astrum.blobstore.api.metrics.BlobStoreMetricsEntity;
import org.globalart.astrum.blobstore.api.metrics.BlobStoreMetricsStore;
import org.globalart.astrum.blobstore.file.FileBlobStore;
import org.globalart.astrum.blobstore.metrics.DatastoreBlobStoreMetricsServiceSupport;
import org.globalart.astrum.common.scheduling.PeriodicJobService;

import com.google.common.collect.ImmutableMap;
import org.springframework.core.Ordered;
import org.springframework.core.annotation.Order;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

@Component
@Qualifier(FileBlobStore.TYPE)
@Order(Ordered.HIGHEST_PRECEDENCE)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class DatastoreFileBlobStoreMetricsService
    extends DatastoreBlobStoreMetricsServiceSupport<FileBlobStore>
{
  @Autowired
  public DatastoreFileBlobStoreMetricsService(
      @Value("${astrum.blobstore.metrics.flushInterval:2}") final int metricsFlushPeriodSeconds,
      final BlobStoreMetricsStore blobStoreMetricsStore,
      final PeriodicJobService jobService)
  {
    super(metricsFlushPeriodSeconds, jobService, blobStoreMetricsStore);
  }

  @Override
  public BlobStoreMetrics getMetrics() {
    BlobStoreMetricsEntity metricsEntity =
        blobStoreMetricsStore.get(blobStore.getBlobStoreConfiguration().getName());

    if (metricsEntity == null) {
      metricsEntity = new BlobStoreMetricsEntity();
    }

    try {
      FileStore fileStore = Files.getFileStore(blobStore.getAbsoluteBlobDir());

      ImmutableMap<String, Long> availableSpace = ImmutableMap
          .of("fileStore:" + fileStore.name(), fileStore.getUsableSpace());

      return new AccumulatingBlobStoreMetrics(
          metricsEntity.getBlobCount(),
          metricsEntity.getTotalSize(),
          availableSpace,
          false);
    }
    catch (IOException e) {
      throw new UncheckedIOException(e);
    }
  }
}
