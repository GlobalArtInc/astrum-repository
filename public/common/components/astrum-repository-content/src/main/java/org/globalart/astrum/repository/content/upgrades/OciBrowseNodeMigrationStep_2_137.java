
package org.globalart.astrum.repository.content.upgrades;

import java.sql.Connection;
import java.sql.Statement;
import java.util.Optional;

import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;

import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

/**
 * OCI-specific migration step: Creates necessary indexes for OCI browse_node table.
 * Drops old index and creates new composite index for efficient browse operations.
 *
 * This migration step is specific to OCI format repository support (ASTRUM-42721).
 * Adds necessary indexes for efficient OCI browse node queries.
 */
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class OciBrowseNodeMigrationStep_2_137
    implements DatabaseMigrationStep
{
  private static final String OCI_FORMAT = "oci";

  private static final String OCI_BROWSE_NODE_TABLE = OCI_FORMAT + "_browse_node";

  private static final String DROP_INDEX = "DROP INDEX IF EXISTS idx_%s_browse_node_asset_id_component_id";

  private static final String CREATE_ASSET_INDEX =
      "CREATE INDEX IF NOT EXISTS idx_%s_browse_node_asset_id_component_id ON %s_browse_node (asset_id, component_id)";

  @Override
  public Optional<String> version() {
    return Optional.of("2.137");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    if (!tableExists(connection, OCI_BROWSE_NODE_TABLE)) {
      return;
    }

    try (Statement st = connection.createStatement()) {
      st.execute(String.format(DROP_INDEX, OCI_FORMAT));
      st.execute(String.format(CREATE_ASSET_INDEX, OCI_FORMAT, OCI_FORMAT));
    }
  }
}
