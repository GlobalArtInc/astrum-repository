
package org.globalart.astrum.repository.search.sql.store.upgrade;

import java.sql.Connection;

import org.globalart.astrum.common.app.FeatureFlags;
import org.globalart.astrum.repository.content.search.upgrade.SearchIndexUpgrade;
import org.globalart.astrum.upgrade.datastore.RepeatableDatabaseMigrationStep;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

/**
 * Triggers the creation of the search index for single node instances after the removal of ElasticSearch.
 */
@Component
public class ReindexSearchTableUpgradeStep
    extends SearchIndexUpgrade
    implements RepeatableDatabaseMigrationStep
{
  private final boolean haEnabled;

  @Autowired
  public ReindexSearchTableUpgradeStep(
      @Value(FeatureFlags.DATASTORE_CLUSTERED_ENABLED_NAMED_VALUE) final boolean haEnabled)
  {
    this.haEnabled = haEnabled;
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    if (haEnabled) {
      log.debug("HA enabled no reindex required");
    }
    else {
      super.migrate(connection);
    }
  }

  @Override
  public Integer getChecksum() {
    return 0;
  }
}
