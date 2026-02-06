
package org.globalart.astrum.repository.content.search.capability;

import java.sql.Connection;
import java.sql.PreparedStatement;
import java.util.Optional;

import org.globalart.astrum.repository.content.search.upgrade.SearchIndexUpgrade;
import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;
import org.springframework.stereotype.Component;

/**
 * Migration step which removes the defunct SearchConfigurationCapability
 */
@Component
public class SearchCapabilityStep_2_16
    extends SearchIndexUpgrade
    implements DatabaseMigrationStep
{
  @Override
  public Optional<String> version() {
    return Optional.of("2.16");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    if (this.tableExists(connection, "capability_storage_item")) {
      try (PreparedStatement statement =
          connection.prepareStatement("DELETE FROM capability_storage_item WHERE type = ?")) {
        statement.setString(1, "astrum.search.configuration");
        statement.execute();
      }
    }

    if (this.tableExists(connection, "search_components")) {
      // trigger re-index
      super.migrate(connection);
    }
  }
}
