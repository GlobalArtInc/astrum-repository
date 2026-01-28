
package org.globalart.astrum.blobstore.metrics.reconcile;

import java.sql.Connection;

import org.globalart.astrum.blobstore.api.metrics.BlobStoreMetricsEntity;
import org.globalart.astrum.repository.content.blobstore.metrics.BlobStoreMetricsDAO;
import org.globalart.astrum.scheduling.TaskConfiguration;
import org.globalart.astrum.scheduling.UpgradeTaskScheduler;
import org.globalart.astrum.testdb.DataSessionConfiguration;
import org.globalart.astrum.testdb.DatabaseTest;
import org.globalart.astrum.testdb.TestDataSessionSupplier;

import org.mockito.InjectMocks;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.times;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.verifyNoInteractions;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;

@ExtendWith(MockitoExtension.class)
class ScheduleRecalculateForNegativeMetricsMigrationStepTest

{
  @DataSessionConfiguration(daos = {})
  TestDataSessionSupplier dataSessionSupplier;

  @Mock
  private UpgradeTaskScheduler upgradeTaskScheduler;

  @InjectMocks
  private ScheduleRecalculateForNegativeMetricsMigrationStep underTest;

  @DatabaseTest
  void testMigrate_withMultipleNegativeMetrics() throws Exception {
    createTable();
    createMetricsRecord("blob-store-1", -1000L, 100L);
    createMetricsRecord("blob-store-2", 5000L, -50L);
    createMetricsRecord("blob-store-3", -1000L, -100L);
    createMetricsRecord("blob-store-4", 10000L, 200L); // positive, should not schedule

    TaskConfiguration taskConfiguration1 = mock();
    TaskConfiguration taskConfiguration2 = mock();
    TaskConfiguration taskConfiguration3 = mock();
    when(upgradeTaskScheduler.createTaskConfigurationInstance(RecalculateBlobStoreSizeTaskDescriptor.TYPE_ID))
        .thenReturn(taskConfiguration1, taskConfiguration2, taskConfiguration3);

    try (Connection connection = dataSessionSupplier.openConnection(DEFAULT_DATASTORE_NAME)) {
      underTest.migrate(connection);
    }

    verify(upgradeTaskScheduler, times(3))
        .createTaskConfigurationInstance(RecalculateBlobStoreSizeTaskDescriptor.TYPE_ID);
    verify(taskConfiguration1).setString(eq("blobstoreName"), eq("blob-store-1"));
    verify(taskConfiguration2).setString(eq("blobstoreName"), eq("blob-store-2"));
    verify(taskConfiguration3).setString(eq("blobstoreName"), eq("blob-store-3"));
    verify(upgradeTaskScheduler).schedule(taskConfiguration1);
    verify(upgradeTaskScheduler).schedule(taskConfiguration2);
    verify(upgradeTaskScheduler).schedule(taskConfiguration3);
  }

  @DatabaseTest
  void testMigrate_noTable() throws Exception {
    try (Connection connection = dataSessionSupplier.openConnection(DEFAULT_DATASTORE_NAME)) {
      underTest.migrate(connection);
    }
    verifyNoInteractions(upgradeTaskScheduler);
  }

  @DatabaseTest
  void testMigrate_emptyTable() throws Exception {
    createTable();

    try (Connection connection = dataSessionSupplier.openConnection(DEFAULT_DATASTORE_NAME)) {
      underTest.migrate(connection);
    }
    verifyNoInteractions(upgradeTaskScheduler);
  }

  private void createMetricsRecord(final String blobStoreName, final long totalSize, final long blobCount) {
    dataSessionSupplier.callDAO(BlobStoreMetricsDAO.class, dao -> {
      dao.initializeMetrics(blobStoreName);
      dao.updateMetrics(
          new BlobStoreMetricsEntity().setBlobStoreName(blobStoreName).setBlobCount(blobCount).setTotalSize(totalSize));
    });
  }

  private void createTable() {
    dataSessionSupplier.register(BlobStoreMetricsDAO.class);
  }
}
