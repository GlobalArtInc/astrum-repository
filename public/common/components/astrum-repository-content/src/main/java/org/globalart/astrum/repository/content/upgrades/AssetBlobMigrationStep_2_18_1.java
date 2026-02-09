
package org.globalart.astrum.repository.content.upgrades;

import java.sql.Connection;
import java.util.Optional;

import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import org.springframework.stereotype.Component;

/**
 * This migration step is now a no-op. Index creation has been moved to a scheduled task
 * (CreateAssetBlobIndexTask) which is scheduled by AssetBlobMigrationStep_2_18_2.
 *
 * Note that the 2_18_1 version is because this step is needed in a 3.86.1 patch release and there are already
 * like 20 new db migrations on main for the 3.87 release, so want to make sure we don't cause migration issues for
 * customers that go to the 3.86.1 release and then 3.87 or any other future release
 */
@Component
public class AssetBlobMigrationStep_2_18_1
    implements DatabaseMigrationStep
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  @Override
  public Optional<String> version() {
    return Optional.of("2.18.1");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    // This migration step is now a no-op. Index creation is handled by the scheduled task
    // in AssetBlobMigrationStep_2_18_2 which schedules CreateAssetBlobIndexTask.
    log.info("AssetBlobMigrationStep_2_18_1 is now a no-op. Index creation is handled by scheduled task.");
  }
}
