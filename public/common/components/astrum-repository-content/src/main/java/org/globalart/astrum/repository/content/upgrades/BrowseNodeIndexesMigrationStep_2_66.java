
package org.globalart.astrum.repository.content.upgrades;

import java.sql.Connection;
import java.sql.SQLException;
import java.util.List;
import java.util.Optional;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.repository.Format;
import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;

import org.springframework.stereotype.Component;

/**
 * Adds indexes to {format}_browse_node tables for existing databases.
 *
 * Related to ASTRUM-49154 - Migration created to allow removal of CREATE INDEX
 * from MyBatis createSchema() which could cause lock contention on startup.
 *
 * @since 3.87
 */
@Component
public class BrowseNodeIndexesMigrationStep_2_66
    implements DatabaseMigrationStep
{
  private final List<Format> formats;

  @Autowired
  public BrowseNodeIndexesMigrationStep_2_66(final List<Format> formats) {
    this.formats = formats;
  }

  @Override
  public Optional<String> version() {
    return Optional.of("2.66");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    for (Format format : formats) {
      String tableName = format.getValue() + "_browse_node";
      if (tableExists(connection, tableName)) {
        createIndexes(connection, format.getValue(), tableName);
      }
    }
  }

  private void createIndexes(
      final Connection connection,
      final String format,
      final String tableName) throws SQLException
  {
    createUniqueIndexIfNotExists(connection, "uk_" + format + "_browse_node_parent_display",
        tableName, "repository_id, parent_id, display_name");

    createIndexIfNotExists(connection, "idx_" + format + "_browse_node_asset_id",
        tableName, "asset_id");

    createIndexIfNotExists(connection, "idx_" + format + "_browse_node_component_id",
        tableName, "component_id");
  }

  private void createUniqueIndexIfNotExists(
      final Connection connection,
      final String indexName,
      final String tableName,
      final String columns) throws SQLException
  {
    if (indexExists(connection, tableName, indexName)) {
      return;
    }

    if (isPostgresql(connection)) {
      runStatement(connection, String.format("""
          DO $$
          BEGIN
            BEGIN
              CREATE UNIQUE INDEX %s ON %s (%s);
            EXCEPTION
              WHEN duplicate_table THEN NULL;
            END;
          END $$;
          """, indexName, tableName, columns));
    }
    else if (isH2(connection)) {
      runStatement(connection,
          String.format("CREATE UNIQUE INDEX IF NOT EXISTS %s ON %s (%s)", indexName, tableName, columns));
    }
  }

  private void createIndexIfNotExists(
      final Connection connection,
      final String indexName,
      final String tableName,
      final String columns) throws SQLException
  {
    if (indexExists(connection, tableName, indexName)) {
      return;
    }

    if (isPostgresql(connection)) {
      runStatement(connection, String.format("""
          DO $$
          BEGIN
            BEGIN
              CREATE INDEX %s ON %s (%s);
            EXCEPTION
              WHEN duplicate_table THEN NULL;
            END;
          END $$;
          """, indexName, tableName, columns));
    }
    else if (isH2(connection)) {
      runStatement(connection,
          String.format("CREATE INDEX IF NOT EXISTS %s ON %s (%s)", indexName, tableName, columns));
    }
  }
}
