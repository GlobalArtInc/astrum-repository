
package org.globalart.astrum.repository.content.upgrades;

import java.sql.Connection;
import java.util.Optional;

import org.globalart.astrum.repository.content.tasks.CreateAssetBlobIndexTaskDescriptor;
import org.globalart.astrum.scheduling.UpgradeTaskScheduler;
import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

/**
 * Schedules the CreateAssetBlobIndexTask to create composite indexes on {format}_asset_blob tables
 * for blob_created DESC and asset_blob_id ASC. This replaces the direct index creation from
 * AssetBlobMigrationStep_2_18_1 which is now a no-op.
 */
@Component
public class AssetBlobMigrationStep_2_18_2
    implements DatabaseMigrationStep
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final UpgradeTaskScheduler upgradeTaskScheduler;

  @Autowired
  public AssetBlobMigrationStep_2_18_2(final UpgradeTaskScheduler upgradeTaskScheduler) {
    this.upgradeTaskScheduler = checkNotNull(upgradeTaskScheduler);
  }

  @Override
  public Optional<String> version() {
    return Optional.of("2.18.2");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    log.info("Scheduling asset blob index creation task");
    upgradeTaskScheduler.schedule(
        upgradeTaskScheduler.createTaskConfigurationInstance(CreateAssetBlobIndexTaskDescriptor.TYPE_ID));
  }
}
