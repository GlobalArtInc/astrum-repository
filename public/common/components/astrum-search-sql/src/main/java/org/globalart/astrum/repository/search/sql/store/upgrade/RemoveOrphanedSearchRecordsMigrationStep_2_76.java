
package org.globalart.astrum.repository.search.sql.store.upgrade;

import java.sql.Connection;
import java.util.Optional;

import org.globalart.astrum.repository.search.sql.store.upgrade.task.CleanOrphanedSearchRecordsTask;
import org.globalart.astrum.scheduling.UpgradeTaskScheduler;
import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Removes orphaned records from search_components and search_assets tables.
 */
@Component
public class RemoveOrphanedSearchRecordsMigrationStep_2_76
    implements DatabaseMigrationStep
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  final UpgradeTaskScheduler taskScheduler;

  @Autowired
  public RemoveOrphanedSearchRecordsMigrationStep_2_76(final UpgradeTaskScheduler taskScheduler) {
    this.taskScheduler = checkNotNull(taskScheduler);
  }

  @Override
  public Optional<String> version() {
    return Optional.of("2.76");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    boolean searchComponentsExists = tableExists(connection, "search_components");
    boolean searchAssetsExists = tableExists(connection, "search_assets");

    if (searchComponentsExists && searchAssetsExists) {
      log.info("scheduling cleanup for search orphaned records");
      taskScheduler.schedule(taskScheduler.createTaskConfigurationInstance(CleanOrphanedSearchRecordsTask.TYPE_ID));
    }
  }
}
