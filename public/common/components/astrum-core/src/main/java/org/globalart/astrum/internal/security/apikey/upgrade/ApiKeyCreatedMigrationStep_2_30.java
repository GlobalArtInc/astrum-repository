
package org.globalart.astrum.internal.security.apikey.upgrade;

import java.sql.Connection;
import java.sql.SQLException;
import java.util.Optional;

import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;

import org.springframework.stereotype.Component;

/**
 * Adds the created column to api_key table.
 *
 * This column tracks when API keys are created, used for auditing and
 * expiration policies.
 *
 * Related to ASTRUM-49154 - Migration created to allow removal of ALTER TABLE
 * from MyBatis createSchema() which was causing lock contention on startup.
 *
 * @since 3.87
 */
@Component
public class ApiKeyCreatedMigrationStep_2_30
    implements DatabaseMigrationStep
{
  private static final String TABLE_NAME = "api_key";

  private static final String COLUMN_NAME = "created";

  @Override
  public Optional<String> version() {
    return Optional.of("2.30");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    if (tableExists(connection, TABLE_NAME)) {
      addColumnIfNotExists(connection);
    }
  }

  private void addColumnIfNotExists(final Connection connection) throws SQLException {
    if (columnExists(connection, TABLE_NAME, COLUMN_NAME)) {
      return; // Column already exists
    }

    if (isPostgresql(connection)) {
      // Use exception handling to avoid lock contention (ASTRUM-49154)
      runStatement(connection, """
          DO $$
          BEGIN
            BEGIN
              ALTER TABLE api_key ADD COLUMN created TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP;
            EXCEPTION
              WHEN duplicate_column THEN NULL;
            END;
          END $$;
          """);
    }
    else if (isH2(connection)) {
      // H2 can safely use IF NOT EXISTS
      runStatement(connection,
          "ALTER TABLE api_key ADD COLUMN IF NOT EXISTS created TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP");
    }
  }
}
