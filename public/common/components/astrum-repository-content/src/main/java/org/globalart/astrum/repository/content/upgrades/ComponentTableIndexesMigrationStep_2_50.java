
package org.globalart.astrum.repository.content.upgrades;

import java.sql.Connection;
import java.sql.SQLException;
import java.util.List;
import java.util.Optional;

import org.globalart.astrum.repository.Format;
import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;

import com.google.common.annotations.VisibleForTesting;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

/**
 * Adds indexes to {format}_component tables for existing databases.
 *
 * Related to ASTRUM-49154 - Migration created to allow removal of CREATE INDEX
 * from MyBatis createSchema() which could cause lock contention on startup.
 *
 * Creates 2 indexes per format:
 * - idx_{format}_component_kind
 * - idx_{format}_normalized_version
 *
 * @since 3.87
 */
@Component
public class ComponentTableIndexesMigrationStep_2_50
    implements DatabaseMigrationStep
{
  private final List<Format> formats;

  @Autowired
  public ComponentTableIndexesMigrationStep_2_50(final List<Format> formats) {
    this.formats = formats;
  }

  @Override
  public Optional<String> version() {
    return Optional.of("2.50");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    for (Format format : formats) {
      String tableName = format.getValue() + "_component";
      if (tableExists(connection, tableName)) {
        createIndexes(connection, format.getValue(), tableName);
      }
    }
  }

  @VisibleForTesting
  void createIndexes(final Connection connection, final String format, final String tableName) throws SQLException {
    // Index 1: kind column
    String kindIndexName = "idx_" + format + "_component_kind";
    createIndexIfNotExists(connection, kindIndexName, tableName, "kind");

    // Index 2: normalized_version column
    String normalizedVersionIndexName = "idx_" + format + "_normalized_version";
    createIndexIfNotExists(connection, normalizedVersionIndexName, tableName, "normalized_version");
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
