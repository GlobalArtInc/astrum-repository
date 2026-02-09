
package org.globalart.astrum.repository.content.upgrades;

import java.sql.Connection;
import java.sql.PreparedStatement;
import java.sql.SQLException;
import java.util.List;
import java.util.Optional;
import java.util.stream.Collectors;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.content.browse.RebuildBrowseNodesManager;
import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;
import org.globalart.astrum.upgrade.datastore.UpgradeContext;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Resolves an issue where multiple browse nodes were created with the same parent and the same display name.
 *
 * @since 3.33
 */
@Component
public class BrowseNodeMigrationStep_1_1
    implements DatabaseMigrationStep
{
  private static final String TRUNCATE = "DELETE FROM {format}_browse_node";

  private static final String TABLE_NAME = "{format}_browse_node";

  private final UpgradeContext upgradeContext;

  private final List<String> formats;

  @Autowired
  public BrowseNodeMigrationStep_1_1(
      final List<Format> formats,
      final UpgradeContext upgradeContext)
  {
    this.formats = formats.stream().map(Format::getValue).collect(Collectors.toList());
    this.upgradeContext = checkNotNull(upgradeContext);
  }

  @Override
  public Optional<String> version() {
    return Optional.of("1.1");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    for (String format : formats) {
      migrate(format, connection);
    }
    // Signals RebuildBrowseNodesManager (TASKS phase) to rebuild browse nodes when it starts.
    upgradeContext.setFlag(RebuildBrowseNodesManager.REBUILD_BROWSE_NODES_ON_START);
  }

  private void migrate(final String formatName, final Connection conn) throws SQLException {
    if (!tableExists(conn, format(TABLE_NAME, formatName))) {
      return;
    }

    try (PreparedStatement select = conn.prepareStatement(format(TRUNCATE, formatName))) {
      select.executeUpdate();
    }
  }

  private static String format(final String query, final String format) {
    return query.replaceAll("\\{format\\}", format);
  }
}
