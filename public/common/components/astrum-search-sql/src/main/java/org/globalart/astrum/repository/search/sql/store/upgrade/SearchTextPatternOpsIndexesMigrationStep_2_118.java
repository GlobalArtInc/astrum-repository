
package org.globalart.astrum.repository.search.sql.store.upgrade;

import java.sql.Connection;
import java.util.Optional;

import org.globalart.astrum.repository.search.sql.store.upgrade.task.CreateSearchTextPatternOpsIndexTaskDescriptor;
import org.globalart.astrum.scheduling.UpgradeTaskScheduler;
import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

/**
 * Schedules the {@link org.globalart.astrum.repository.search.sql.store.upgrade.task.CreateSearchTextPatternOpsIndexTask}
 * to create text_pattern_ops indexes on search_components columns.
 */
@Component
public class SearchTextPatternOpsIndexesMigrationStep_2_118
    implements DatabaseMigrationStep
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final UpgradeTaskScheduler upgradeTaskScheduler;

  @Autowired
  public SearchTextPatternOpsIndexesMigrationStep_2_118(final UpgradeTaskScheduler upgradeTaskScheduler) {
    this.upgradeTaskScheduler = checkNotNull(upgradeTaskScheduler);
  }

  @Override
  public Optional<String> version() {
    return Optional.of("2.118");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    log.info("Scheduling text_pattern_ops index creation task");
    upgradeTaskScheduler.schedule(
        upgradeTaskScheduler.createTaskConfigurationInstance(CreateSearchTextPatternOpsIndexTaskDescriptor.TYPE_ID));
  }
}
