
package org.globalart.astrum.blobstore.metrics.reconcile;

import java.sql.Connection;
import java.sql.PreparedStatement;
import java.sql.ResultSet;
import java.util.ArrayList;
import java.util.List;

import org.globalart.astrum.blobstore.common.BlobStoreTaskSupport;
import org.globalart.astrum.scheduling.TaskConfiguration;
import org.globalart.astrum.scheduling.UpgradeTaskScheduler;
import org.globalart.astrum.upgrade.datastore.RepeatableDatabaseMigrationStep;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Schedules {@link RecalculateBlobStoreSizeTask} for blob stores with negative count or size metrics.
 */
@Component
public class ScheduleRecalculateForNegativeMetricsMigrationStep
    implements RepeatableDatabaseMigrationStep
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private static final String TABLE = "blob_store_metrics";

  private static final String SELECT_NEGATIVE_METRICS = """
      SELECT blob_store_name
        FROM %s
       WHERE total_size < 0 OR blob_count < 0;
      """.formatted(TABLE);

  private final UpgradeTaskScheduler upgradeTaskScheduler;

  @Autowired
  public ScheduleRecalculateForNegativeMetricsMigrationStep(final UpgradeTaskScheduler upgradeTaskScheduler) {
    this.upgradeTaskScheduler = checkNotNull(upgradeTaskScheduler);
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    if (!tableExists(connection, TABLE)) {
      log.debug("Table {} does not exist, skipping migration", TABLE);
      return;
    }

    List<String> blobStoresWithNegativeMetrics = new ArrayList<>();

    try (PreparedStatement statement = connection.prepareStatement(SELECT_NEGATIVE_METRICS);
        ResultSet results = statement.executeQuery()) {
      while (results.next()) {
        String blobStoreName = results.getString("blob_store_name");
        blobStoresWithNegativeMetrics.add(blobStoreName);
        log.info("Found blob store '{}' with negative metrics, scheduling recalculation", blobStoreName);
      }
    }

    if (blobStoresWithNegativeMetrics.isEmpty()) {
      log.debug("No blob stores with negative metrics found");
      return;
    }

    // Schedule a task for each blob store with negative metrics
    for (String blobStoreName : blobStoresWithNegativeMetrics) {
      TaskConfiguration configuration = upgradeTaskScheduler.createTaskConfigurationInstance(
          RecalculateBlobStoreSizeTaskDescriptor.TYPE_ID);
      configuration.setString(BlobStoreTaskSupport.BLOBSTORE_NAME_FIELD_ID, blobStoreName);
      upgradeTaskScheduler.schedule(configuration);
    }

    log.info("Scheduled recalculation tasks for {} blob store(s) with negative metrics",
        blobStoresWithNegativeMetrics.size());
  }

  @Override
  public Integer getChecksum() {
    return 1;
  }
}
