
package org.globalart.astrum.repository.content.upgrades;

import java.sql.Connection;
import java.util.Optional;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.repository.content.tasks.CreateComponentIndexTaskDescriptor;
import org.globalart.astrum.scheduling.UpgradeTaskScheduler;
import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;

import org.slf4j.LoggerFactory;
import org.slf4j.Logger;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

@Component
public class CoordinateMigrationStep_2_13
    implements DatabaseMigrationStep
{
  private static final Logger LOG = LoggerFactory.getLogger(CoordinateMigrationStep_2_13.class);

  private final UpgradeTaskScheduler upgradeTaskScheduler;

  @Autowired
  public CoordinateMigrationStep_2_13(final UpgradeTaskScheduler upgradeTaskScheduler) {
    this.upgradeTaskScheduler = checkNotNull(upgradeTaskScheduler);
  }

  @Override
  public Optional<String> version() {
    return Optional.of("2.13");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    LOG.info("Scheduling CreateComponentIndexTask to run after upgrade is completed.");
    upgradeTaskScheduler.schedule(upgradeTaskScheduler.createTaskConfigurationInstance(
        CreateComponentIndexTaskDescriptor.TYPE_ID));
  }
}
