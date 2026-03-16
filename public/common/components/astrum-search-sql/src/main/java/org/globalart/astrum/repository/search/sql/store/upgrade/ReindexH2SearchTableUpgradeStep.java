
package org.globalart.astrum.repository.search.sql.store.upgrade;

import java.sql.Connection;

import org.globalart.astrum.repository.content.search.upgrade.SearchIndexUpgrade;
import org.globalart.astrum.upgrade.datastore.RepeatableDatabaseMigrationStep;

import org.springframework.stereotype.Component;

/**
 * Triggers the recreation of the search index for H2 instances due to a table schema change introduced by
 * {@link SearchTablePathsColumnJsonMigrationStep_2_102}.
 */
@Component
public class ReindexH2SearchTableUpgradeStep
    extends SearchIndexUpgrade
    implements RepeatableDatabaseMigrationStep
{
  @Override
  public void migrate(final Connection connection) throws Exception {
    if (isH2(connection)) {
      log.info("Reindexing H2 search tables");
      super.migrate(connection);
    }
    else {
      log.debug("Skipping re-index because instance does not use H2");
    }
  }

  @Override
  public Integer getChecksum() {
    return 0;
  }
}
