
package org.globalart.astrum.repository.search.sql.store.upgrade;

import java.sql.Connection;
import java.util.Optional;

import org.globalart.astrum.common.app.FeatureFlags;
import org.globalart.astrum.scheduling.TaskConfiguration;
import org.globalart.astrum.scheduling.TaskScheduler;
import org.globalart.astrum.scheduling.UpgradeTaskScheduler;
import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.search.sql.store.upgrade.task.ReIndexSearchFilterByPatternTask.FILTER_CONDITION_FIELD_ID;
import static org.globalart.astrum.repository.search.sql.store.upgrade.task.ReIndexSearchFilterByPatternTask.TYPE_ID;

/**
 * Schedules a reindex task which filters components that contains underscores for namespace or version fields.
 */
@Component
public class ReIndexSearchRecordsWithUnderscoreMigrationStep_2_105
    implements DatabaseMigrationStep
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final TaskScheduler taskScheduler;

  private final UpgradeTaskScheduler upgradeTaskScheduler;

  private final boolean haEnabled;

  @Autowired
  public ReIndexSearchRecordsWithUnderscoreMigrationStep_2_105(
      TaskScheduler taskScheduler,
      UpgradeTaskScheduler upgradeTaskScheduler,
      @Value(FeatureFlags.DATASTORE_CLUSTERED_ENABLED_NAMED_VALUE) final boolean haEnabled)
  {
    this.haEnabled = haEnabled;
    this.taskScheduler = checkNotNull(taskScheduler);
    this.upgradeTaskScheduler = checkNotNull(upgradeTaskScheduler);
  }

  @Override
  public Optional<String> version() {
    return Optional.of("2.105");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    if (haEnabled) {
      scheduleReindexTaskForRecordsWithUnderscore();
    }
    else {
      log.debug("HA not enabled no need to schedule the reindex with filter task");
    }
  }

  private void scheduleReindexTaskForRecordsWithUnderscore() {
    TaskConfiguration taskConfiguration = taskScheduler.createTaskConfigurationInstance(TYPE_ID);
    taskConfiguration.setString(FILTER_CONDITION_FIELD_ID, "namespace LIKE '%\\_%' OR version LIKE '%\\_%'");
    upgradeTaskScheduler.schedule(taskConfiguration);
    log.debug("Scheduled reindex with filter task");
  }
}
