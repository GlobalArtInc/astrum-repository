
package org.globalart.astrum.repository.apt.internal.upgrade;

import java.sql.Connection;
import java.util.Optional;

import org.globalart.astrum.repository.RepositoryTaskSupport;
import org.globalart.astrum.scheduling.TaskConfiguration;
import org.globalart.astrum.scheduling.UpgradeTaskScheduler;
import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Start task to repair Apt metadata adding leading slash to Filename
 */
@Component
public class RepairAptMetadataLeadingSlash_2_22
    implements DatabaseMigrationStep
{
  private final UpgradeTaskScheduler taskScheduler;

  @Autowired
  public RepairAptMetadataLeadingSlash_2_22(final UpgradeTaskScheduler taskScheduler) {
    this.taskScheduler = checkNotNull(taskScheduler);
  }

  @Override
  public Optional<String> version() {
    return Optional.of("2.22");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    TaskConfiguration configuration =
        taskScheduler.createTaskConfigurationInstance(RepairAptMetadataLeadingSlashTaskDescriptor.TYPE_ID);
    configuration.setString(RepositoryTaskSupport.REPOSITORY_NAME_FIELD_ID, RepositoryTaskSupport.ALL_REPOSITORIES);
    taskScheduler.schedule(configuration);
  }
}
