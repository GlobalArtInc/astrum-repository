
package org.globalart.astrum.internal.log.overrides.datastore.upgrade;

import java.sql.Connection;
import java.sql.SQLException;
import java.util.Optional;

import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;

import org.springframework.stereotype.Component;

/**
 * Adds unique index to logging_overrides table for existing databases.
 *
 * Related to ASTRUM-49154 - Migration created to allow removal of CREATE INDEX
 * from MyBatis createSchema() which could cause lock contention on startup.
 *
 * @since 3.87
 */
@Component
public class LoggingOverridesIndexMigrationStep_2_44
    implements DatabaseMigrationStep
{
  private static final String TABLE_NAME = "logging_overrides";

  @Override
  public Optional<String> version() {
    return Optional.of("2.44");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    if (tableExists(connection, TABLE_NAME)) {
      createUniqueIndexIfNotExists(connection, "uk_name", TABLE_NAME, "name");
    }
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
}
