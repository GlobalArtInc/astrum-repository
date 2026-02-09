
package org.globalart.astrum.repository.content.upgrades;

import java.sql.Connection;
import java.util.Optional;

import org.globalart.astrum.repository.content.tasks.BlobRepositoryMismatchTaskDescriptor;
import org.globalart.astrum.scheduling.TaskConfiguration;
import org.globalart.astrum.scheduling.UpgradeTaskScheduler;
import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.RepositoryTaskSupport.ALL_REPOSITORIES;
import static org.globalart.astrum.repository.RepositoryTaskSupport.REPOSITORY_NAME_FIELD_ID;

/**
 * Fixes any mismatch between the repository the blob is currently in and the Bucket.repo-name in the properties file
 */
@Component
public class BlobRepositoryMismatchMigrationStep_2_106
    implements DatabaseMigrationStep
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final UpgradeTaskScheduler upgradeTaskScheduler;

  @Autowired
  public BlobRepositoryMismatchMigrationStep_2_106(final UpgradeTaskScheduler upgradeTaskScheduler) {
    this.upgradeTaskScheduler = checkNotNull(upgradeTaskScheduler);
  }

  @Override
  public Optional<String> version() {
    return Optional.of("2.106");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    log.info("Scheduling blob repository mismatch task");
    TaskConfiguration taskConfiguration = upgradeTaskScheduler
        .createTaskConfigurationInstance(BlobRepositoryMismatchTaskDescriptor.TYPE_ID);
    taskConfiguration.setString(REPOSITORY_NAME_FIELD_ID, ALL_REPOSITORIES);
    upgradeTaskScheduler.schedule(taskConfiguration, false);
    log.info("Scheduled blob repository mismatch task for all repositories");
  }
}
