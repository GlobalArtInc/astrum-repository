
package org.globalart.astrum.internal.security.secrets.upgrade;

import java.sql.Connection;
import java.util.Optional;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.internal.security.secrets.task.SecretsMigrationTaskDescriptor;
import org.globalart.astrum.scheduling.UpgradeTaskScheduler;
import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.crypto.secrets.SecretsService.SECRETS_MIGRATION_VERSION;
import org.springframework.stereotype.Component;

/**
 * Database migration step to migrate existing secrets to secrets table using new encryption implementation (Random
 * IV/Salt & custom encryption key)
 */
@Component
public class SecretsMigrationStep_2_2
    implements DatabaseMigrationStep
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final UpgradeTaskScheduler startupScheduler;

  @Autowired
  public SecretsMigrationStep_2_2(final UpgradeTaskScheduler startupScheduler) {
    this.startupScheduler = checkNotNull(startupScheduler);
  }

  @Override
  public Optional<String> version() {
    return Optional.of(SECRETS_MIGRATION_VERSION);
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    if (tableExists(connection, "secrets")) {
      log.debug("starting secrets migration task");
      startupScheduler.schedule(
          startupScheduler.createTaskConfigurationInstance(SecretsMigrationTaskDescriptor.TYPE_ID));
    }
  }
}
