
package org.globalart.astrum.repository.content.upgrades;

import java.sql.Connection;
import java.sql.PreparedStatement;
import java.sql.SQLException;
import java.util.List;
import java.util.Optional;
import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.repository.Format;
import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Remove duplicate, unnecessary index
 * Add additional index for asset querying (similar to component)
 */
@Component
public class BrowseNodeMigrationStep_1_34
    implements DatabaseMigrationStep
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final List<Format> formats;

  @Autowired
  public BrowseNodeMigrationStep_1_34(final List<Format> formats) {
    this.formats = formats;
  }

  private static final String DROP_INDEX = "DROP INDEX IF EXISTS idx_%s_browse_node_tree";

  private static final String CREATE_ASSET_INDEX = "CREATE INDEX IF NOT EXISTS " +
      "idx_%s_browse_node_asset_id ON %s_browse_node (asset_id);";

  @Override
  public Optional<String> version() {
    return Optional.of("1.34");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    formats.forEach(format -> migrateFormat(connection, format));
  }

  private void migrateFormat(final Connection connection, final Format format) {
    String formatName = format.getValue();
    executeStatement(connection, String.format(DROP_INDEX, formatName));
    executeStatement(connection, String.format(CREATE_ASSET_INDEX, formatName, formatName));
  }

  private void executeStatement(final Connection connection, final String sqlStatement) {
    try (PreparedStatement select = connection.prepareStatement(sqlStatement)) {
      select.executeUpdate();
    }
    catch (SQLException e) {
      log.error("Failed to apply browse_node index change ('{}')", sqlStatement, e);
      throw new RuntimeException(e);
    }
  }
}
