
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
import org.springframework.stereotype.Component;

/**
 * Migration step to migrate session token from existing blobstore configurations
 */
@Component
public class BlobstoreSecretsMigrationStep_2_11
    implements DatabaseMigrationStep
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final UpgradeTaskScheduler startupScheduler;

  @Autowired
  public BlobstoreSecretsMigrationStep_2_11(final UpgradeTaskScheduler startupScheduler) {
    this.startupScheduler = checkNotNull(startupScheduler);
  }

  @Override
  public Optional<String> version() {
    return Optional.of("2.11");
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
