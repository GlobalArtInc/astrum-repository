
package org.globalart.astrum.internal.jwt.datastore.upgrade;

import java.sql.Connection;
import java.sql.SQLException;
import java.util.Optional;

import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;

import org.springframework.stereotype.Component;

/**
 * Adds index to jwt_session table for existing databases.
 *
 * Related to ASTRUM-49154 - Migration created to allow removal of CREATE INDEX
 * from MyBatis createSchema() which could cause lock contention on startup.
 */
@Component
public class JwtSessionIndexMigrationStep_2_103
    implements DatabaseMigrationStep
{
  private static final String TABLE_NAME = "jwt_session";

  @Override
  public Optional<String> version() {
    return Optional.of("2.103");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    if (tableExists(connection, TABLE_NAME)) {
      createIndexIfNotExists(connection, "idx_jwt_session_expires",
          TABLE_NAME, "expires_at");
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
