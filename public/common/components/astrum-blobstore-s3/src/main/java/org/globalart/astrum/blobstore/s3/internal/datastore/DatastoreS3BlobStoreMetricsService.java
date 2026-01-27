
package org.globalart.astrum.blobstore.s3.internal.datastore;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.blobstore.AccumulatingBlobStoreMetrics;
import org.globalart.astrum.blobstore.api.BlobStoreMetrics;
import org.globalart.astrum.blobstore.api.metrics.BlobStoreMetricsEntity;
import org.globalart.astrum.blobstore.api.metrics.BlobStoreMetricsStore;
import org.globalart.astrum.blobstore.metrics.DatastoreBlobStoreMetricsServiceSupport;
import org.globalart.astrum.blobstore.s3.internal.S3BlobStore;
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
@Qualifier(S3BlobStore.TYPE)
@Order(Ordered.HIGHEST_PRECEDENCE)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class DatastoreS3BlobStoreMetricsService
    extends DatastoreBlobStoreMetricsServiceSupport<S3BlobStore>
{
  private static final ImmutableMap<String, Long> AVAILABLE_SPACE_BY_FILE_STORE = ImmutableMap.of("s3", Long.MAX_VALUE);

  @Autowired
  public DatastoreS3BlobStoreMetricsService(
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

    return new AccumulatingBlobStoreMetrics(
        metricsEntity.getBlobCount(),
        metricsEntity.getTotalSize(),
        AVAILABLE_SPACE_BY_FILE_STORE,
        true);
  }
}
