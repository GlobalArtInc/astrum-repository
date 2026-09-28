
package org.globalart.astrum.ssl.plugin.upgrades;

import java.sql.Connection;
import java.util.Optional;
import org.springframework.beans.factory.annotation.Autowired;
import com.globalart.astrum.ssl.plugin.tasks.TrustedCertificatesMigrationTaskDescriptor;
import org.globalart.astrum.scheduling.UpgradeTaskScheduler;
import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

@Component
public class TrustedCertificatesMigrationStep_2_17
    implements DatabaseMigrationStep
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final UpgradeTaskScheduler upgradeTaskScheduler;

  @Autowired
  public TrustedCertificatesMigrationStep_2_17(final UpgradeTaskScheduler upgradeTaskScheduler) {
    this.upgradeTaskScheduler = checkNotNull(upgradeTaskScheduler);
  }

  @Override
  public Optional<String> version() {
    return Optional.of("2.17");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    log.info("Scheduling TrustedCertificatesMigrationTask.");
    upgradeTaskScheduler.schedule(
        upgradeTaskScheduler.createTaskConfigurationInstance(TrustedCertificatesMigrationTaskDescriptor.TYPE_ID));
  }
}
