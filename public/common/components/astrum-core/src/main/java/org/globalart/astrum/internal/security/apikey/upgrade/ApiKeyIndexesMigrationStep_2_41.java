
package org.globalart.astrum.internal.security.apikey.upgrade;

import java.sql.Connection;
import java.sql.SQLException;
import java.util.Optional;

import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;

import org.springframework.stereotype.Component;

/**
 * Adds indexes to api_key_v2 table for existing databases.
 *
 * Related to ASTRUM-49154 - Migration created to allow removal of CREATE INDEX
 * from MyBatis createSchema() which could cause lock contention on startup.
 *
 * @since 3.87
 */
@Component
public class ApiKeyIndexesMigrationStep_2_41
    implements DatabaseMigrationStep
{
  private static final String TABLE_NAME = "api_key_v2";

  @Override
  public Optional<String> version() {
    return Optional.of("2.41");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    if (tableExists(connection, TABLE_NAME)) {
      createIndexes(connection);
    }
  }

  private void createIndexes(final Connection connection) throws SQLException {
    createIndexIfNotExists(connection, "idx_api_key_v2_domain_created",
        TABLE_NAME, "domain, created");
    createIndexIfNotExists(connection, "idx_api_key_v2_created",
        TABLE_NAME, "created");
    createIndexIfNotExists(connection, "idx_api_key_v2_principals",
        TABLE_NAME, "principals");
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
