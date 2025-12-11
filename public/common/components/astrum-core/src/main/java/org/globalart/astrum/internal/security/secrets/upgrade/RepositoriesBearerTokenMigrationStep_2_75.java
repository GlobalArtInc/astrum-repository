
package org.globalart.astrum.internal.security.secrets.upgrade;

import java.sql.Connection;
import java.util.Optional;

import org.globalart.astrum.internal.security.secrets.task.RepositoriesBearerTokenConfigMigrationTaskDescriptor;
import org.globalart.astrum.scheduling.UpgradeTaskScheduler;
import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Migration step to encrypt repositories bearer tokens in authentication configurations.
 */
@Component
public class RepositoriesBearerTokenMigrationStep_2_75
    implements DatabaseMigrationStep
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final UpgradeTaskScheduler startupScheduler;

  @Autowired
  public RepositoriesBearerTokenMigrationStep_2_75(final UpgradeTaskScheduler startupScheduler) {
    this.startupScheduler = checkNotNull(startupScheduler);
  }

  @Override
  public Optional<String> version() {
    return Optional.of("2.75");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    if (tableExists(connection, "secrets")) {
      log.debug("Scheduling bearer token configuration migration task for repositories");
      startupScheduler.schedule(
          startupScheduler
              .createTaskConfigurationInstance(RepositoriesBearerTokenConfigMigrationTaskDescriptor.TYPE_ID));
    }
  }
}
