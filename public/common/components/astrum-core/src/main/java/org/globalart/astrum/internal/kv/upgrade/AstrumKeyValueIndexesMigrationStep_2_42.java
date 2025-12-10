
package org.globalart.astrum.internal.kv.upgrade;

import java.sql.Connection;
import java.sql.SQLException;
import java.util.Optional;

import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;

import org.springframework.stereotype.Component;

/**
 * Adds indexes to astrum_key_value table for existing databases.
 *
 * Related to ASTRUM-49154 - Migration created to allow removal of CREATE INDEX
 * from MyBatis createSchema() which could cause lock contention on startup.
 *
 * @since 3.87
 */
@Component
public class AstrumKeyValueIndexesMigrationStep_2_42
    implements DatabaseMigrationStep
{
  private static final String TABLE_NAME = "astrum_key_value";

  @Override
  public Optional<String> version() {
    return Optional.of("2.42");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    if (tableExists(connection, TABLE_NAME)) {
      createIndexes(connection);
    }
  }

  private void createIndexes(final Connection connection) throws SQLException {
    // Note: 'key' is a reserved keyword in H2, so we need to escape it with backticks
    String columnName = isH2(connection) ? "`key`" : "key";

    // Create unique index first
    createUniqueIndexIfNotExists(connection, "uk_astrum_key_value_key",
        TABLE_NAME, columnName);

    // Create regular index
    createIndexIfNotExists(connection, "idx_astrum_key_value_key",
        TABLE_NAME, columnName);
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
