
package org.globalart.astrum.blobstore.internal.metrics;

import java.util.Map;

import org.globalart.astrum.blobstore.AccumulatingBlobStoreMetrics;
import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;
import org.globalart.astrum.blobstore.api.BlobStoreManager;
import org.globalart.astrum.blobstore.api.BlobStoreMetrics;
import org.globalart.astrum.blobstore.api.OperationMetrics;
import org.globalart.astrum.blobstore.api.OperationType;
import org.globalart.astrum.blobstore.api.metrics.BlobStoreMetricsEntity;
import org.globalart.astrum.blobstore.api.metrics.BlobStoreMetricsPropertiesReader;
import org.globalart.astrum.blobstore.api.metrics.BlobStoreMetricsStore;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.context.ApplicationContext;

import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.lenient;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.times;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.verifyNoMoreInteractions;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
public class BlobStoreMetricsMigrationTaskTest
{
  private static final String BEAN_NAME = "MockBlobStoreMetricsPropertiesReader";

  @Mock
  private ApplicationContext context;

  @Mock
  private BlobStoreManager blobStoreManager;

  @Mock
  private BlobStoreMetricsStore metricsStore;

  @Mock
  private BlobStore blobStore;

  @Mock
  private BlobStoreMetricsPropertiesReader<?> propertiesReader;

  private BlobStoreMetrics blobStoreMetrics = new AccumulatingBlobStoreMetrics(1, 1, Map.of("default", 1l), false);

  private OperationMetrics operationMetrics = new OperationMetrics();

  private BlobStoreMetricsMigrationTask task;

  @BeforeEach
  public void setUp() {
    lenient().when(context.getBeanNamesForType(BlobStoreMetricsPropertiesReader.class))
        .thenReturn(new String[]{BEAN_NAME});
    lenient().when(context.getAliases(BEAN_NAME)).thenReturn(new String[]{"default"});
    lenient().when(context.getBean(BEAN_NAME, BlobStoreMetricsPropertiesReader.class)).thenReturn(propertiesReader);
    task = new BlobStoreMetricsMigrationTask(blobStoreManager, metricsStore, context);
  }

  @Test
  public void testExecuteBlobStoreNotStarted() {
    when(blobStore.isStarted()).thenReturn(false);
    BlobStoreConfiguration configuration = mockBlobStoreConfiguration("default", "testBlobStore");
    when(blobStore.getBlobStoreConfiguration()).thenReturn(configuration);

    task.execute(blobStore);

    verify(blobStore, times(1)).isStarted();
    verifyNoMoreInteractions(metricsStore);
  }

  @Test
  public void testExecutePropertiesReaderNotFound() {
    when(blobStore.isStarted()).thenReturn(true);
    BlobStoreConfiguration configuration = mockBlobStoreConfiguration("unknown", "testBlobStore");
    when(blobStore.getBlobStoreConfiguration()).thenReturn(configuration);

    task.execute(blobStore);

    verify(blobStore, times(1)).isStarted();
    verifyNoMoreInteractions(metricsStore);
  }

  @Test
  public void testExecuteSuccessfulMigration() throws Exception {
    when(blobStore.isStarted()).thenReturn(true);
    BlobStoreConfiguration configuration = mockBlobStoreConfiguration("default", "testBlobStore");
    when(blobStore.getBlobStoreConfiguration()).thenReturn(configuration);
    when(metricsStore.get("testBlobStore")).thenReturn(new BlobStoreMetricsEntity());
    when(propertiesReader.getMetrics()).thenReturn(blobStoreMetrics);
    when(propertiesReader.getOperationMetrics()).thenReturn(
        Map.of(OperationType.DOWNLOAD, operationMetrics, OperationType.UPLOAD, operationMetrics));
    task.execute(blobStore);

    verify(blobStore, times(1)).isStarted();
    verify(metricsStore, times(1)).initializeMetrics("testBlobStore");
    verify(metricsStore, times(1)).updateMetrics(any(BlobStoreMetricsEntity.class));
  }

  @Test
  public void testExecuteMetricsFromFileIsNull() throws Exception {
    when(blobStore.isStarted()).thenReturn(true);
    BlobStoreConfiguration configuration = mockBlobStoreConfiguration("default", "testBlobStore");
    when(blobStore.getBlobStoreConfiguration()).thenReturn(configuration);
    when(propertiesReader.getMetrics()).thenReturn(null);
    when(propertiesReader.getOperationMetrics()).thenReturn(
        Map.of(OperationType.DOWNLOAD, operationMetrics, OperationType.UPLOAD, operationMetrics));

    task.execute(blobStore);

    verify(blobStore, times(1)).isStarted();
    verifyNoMoreInteractions(metricsStore);
  }

  @Test
  public void testExecuteOperationMetricsIsNull() throws Exception {
    when(blobStore.isStarted()).thenReturn(true);
    BlobStoreConfiguration configuration = mockBlobStoreConfiguration("default", "testBlobStore");
    when(blobStore.getBlobStoreConfiguration()).thenReturn(configuration);
    when(propertiesReader.getMetrics()).thenReturn(blobStoreMetrics);
    when(propertiesReader.getOperationMetrics()).thenReturn(null);

    task.execute(blobStore);

    verify(blobStore, times(1)).isStarted();
    verifyNoMoreInteractions(metricsStore);
  }

  @Test
  public void testExecuteMetricsFromFileAndOperationMetricsAreNull() throws Exception {
    when(blobStore.isStarted()).thenReturn(true);
    BlobStoreConfiguration configuration = mockBlobStoreConfiguration("default", "testBlobStore");
    when(blobStore.getBlobStoreConfiguration()).thenReturn(configuration);
    when(propertiesReader.getMetrics()).thenReturn(null);
    when(propertiesReader.getOperationMetrics()).thenReturn(null);

    task.execute(blobStore);

    verify(blobStore, times(1)).isStarted();
    verifyNoMoreInteractions(metricsStore);
  }

  private BlobStoreConfiguration mockBlobStoreConfiguration(final String type, final String name) {
    BlobStoreConfiguration configuration = mock(BlobStoreConfiguration.class);
    when(configuration.getType()).thenReturn(type);
    when(configuration.getName()).thenReturn(name);
    return configuration;
  }
}
