
package org.globalart.astrum.blobstore.internal.metrics;

import java.util.Map;
import java.util.Optional;

import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.blobstore.api.BlobStoreManager;
import org.globalart.astrum.blobstore.api.BlobStoreMetrics;
import org.globalart.astrum.blobstore.api.OperationMetrics;
import org.globalart.astrum.blobstore.api.OperationType;
import org.globalart.astrum.blobstore.api.metrics.BlobStoreMetricsEntity;
import org.globalart.astrum.blobstore.api.metrics.BlobStoreMetricsPropertiesReader;
import org.globalart.astrum.blobstore.api.metrics.BlobStoreMetricsStore;
import org.globalart.astrum.blobstore.common.BlobStoreTaskSupport;
import org.globalart.astrum.blobstore.group.BlobStoreGroup;
import org.globalart.astrum.logging.task.TaskLogging;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.NoSuchBeanDefinitionException;
import org.springframework.beans.factory.annotation.BeanFactoryAnnotationUtils;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.ApplicationContext;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.blobstore.api.OperationType.DOWNLOAD;
import static org.globalart.astrum.blobstore.api.OperationType.UPLOAD;
import static org.globalart.astrum.logging.task.TaskLogType.ASTRUM_LOG_ONLY;

@Component
@TaskLogging(ASTRUM_LOG_ONLY)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class BlobStoreMetricsMigrationTask
    extends BlobStoreTaskSupport
{
  public static final String TYPE_ID = "astrum.blobstore.metrics.migration.task";

  private final BlobStoreMetricsStore metricsStore;

  private final ApplicationContext context;

  @Autowired
  public BlobStoreMetricsMigrationTask(
      final BlobStoreManager blobStoreManager,
      final BlobStoreMetricsStore metricsStore,
      final ApplicationContext context)
  {
    super(blobStoreManager);
    this.metricsStore = checkNotNull(metricsStore);
    this.context = checkNotNull(context);
  }

  @Override
  public String getMessage() {
    return "Migration - Move blobstore metrics to the database";
  }

  @Override
  protected boolean appliesTo(final BlobStore blobStore) {
    return !blobStore.getBlobStoreConfiguration().getType().equals(BlobStoreGroup.TYPE);
  }

  @Override
  protected void execute(final BlobStore blobStore) {
    String blobStoreName = blobStore.getBlobStoreConfiguration().getName();
    String blobStoreType = blobStore.getBlobStoreConfiguration().getType();

    if (!blobStore.isStarted()) {
      log.warn("Blob store {}:{} is not started, skipping it.", blobStoreType, blobStoreName);
      return;
    }

    try {
      Optional<BlobStoreMetricsPropertiesReader<BlobStore>> optPropertiesReader = reader(blobStoreType);
      if (optPropertiesReader.isEmpty()) {
        log.error("Properties reader not found for {}:{}", blobStoreType, blobStoreName);
        return;
      }

      BlobStoreMetricsPropertiesReader<BlobStore> propertiesReader = optPropertiesReader.get();
      propertiesReader.init(blobStore);
      BlobStoreMetrics metricsFromFile = propertiesReader.getMetrics();
      Map<OperationType, OperationMetrics> operationMetrics = propertiesReader.getOperationMetrics();

      if (metricsFromFile != null && operationMetrics != null) {
        String blobStoreId = blobStoreType + ":" + blobStoreName;
        BlobStoreMetricsEntity metricsFromDb = metricsStore.get(blobStoreName);
        log.info("Initial db metrics for {} are: {}", blobStoreId, metricsFromDb);
        log.info("Metrics from filesystem to be migrated for {} are: {}", blobStoreId, metricsFromFile);
        operationMetrics.forEach((operationType, opMetrics) -> log.info(
            "Operational metrics from filesystem to be migrated for {} and {} are: {}",
            blobStoreId, operationType, opMetrics));
        metricsStore.initializeMetrics(blobStoreName);
        metricsStore.updateMetrics(toBlobStoreMetricsEntity(blobStoreName, metricsFromFile, operationMetrics));
        log.info("Final db metrics for {} are: {}", blobStoreId, metricsStore.get(blobStoreName));
      }
    }
    catch (Exception e) {
      log.error("Exception during migrating metrics from properties to DB for {}:{}", blobStoreType, blobStoreName, e);
    }
  }

  private Optional<BlobStoreMetricsPropertiesReader<BlobStore>> reader(final String blobStoreType) {
    try {
      return Optional.of(BeanFactoryAnnotationUtils.qualifiedBeanOfType(context, BlobStoreMetricsPropertiesReader.class,
          blobStoreType));
    }
    catch (NoSuchBeanDefinitionException e) {
      return Optional.empty();
    }
  }

  private static BlobStoreMetricsEntity toBlobStoreMetricsEntity(
      final String blobStoreName,
      final BlobStoreMetrics blobStoreMetrics,
      final Map<OperationType, OperationMetrics> operationMetrics)
  {
    OperationMetrics downloadMetrics = operationMetrics.get(DOWNLOAD);
    OperationMetrics uploadMetrics = operationMetrics.get(UPLOAD);

    return new BlobStoreMetricsEntity()
        .setBlobStoreName(blobStoreName)
        .setBlobCount(blobStoreMetrics.getBlobCount())
        .setTotalSize(blobStoreMetrics.getTotalSize())
        .setDownloadBlobSize(downloadMetrics.getBlobSize())
        .setDownloadErrorRequests(downloadMetrics.getErrorRequests())
        .setDownloadSuccessfulRequests(downloadMetrics.getSuccessfulRequests())
        .setDownloadTimeOnRequests(downloadMetrics.getTimeOnRequests())
        .setUploadBlobSize(uploadMetrics.getBlobSize())
        .setUploadErrorRequests(uploadMetrics.getErrorRequests())
        .setUploadSuccessfulRequests(uploadMetrics.getSuccessfulRequests())
        .setUploadTimeOnRequests(uploadMetrics.getTimeOnRequests());
  }
}
