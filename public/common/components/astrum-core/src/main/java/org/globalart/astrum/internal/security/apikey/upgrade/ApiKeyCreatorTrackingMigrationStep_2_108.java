
package org.globalart.astrum.internal.security.apikey.upgrade;

import java.sql.Connection;
import java.util.Optional;

import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;

import org.springframework.stereotype.Component;

/**
 * Adds creator tracking columns to api_key_v2 table for admin token operations.
 *
 * Related to ASTRUM-50483 - Admin User Token Management
 * Related to ASTRUM-50484 - Admin Token Auditing and Governance
 */
@Component
public class ApiKeyCreatorTrackingMigrationStep_2_108
    implements DatabaseMigrationStep
{
  private static final String TABLE_NAME = "api_key_v2";

  @Override
  public Optional<String> version() {
    return Optional.of("2.108");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    if (tableExists(connection, TABLE_NAME)) {
      addCreatorTrackingColumns(connection);
    }
  }

  private void addCreatorTrackingColumns(final Connection connection) throws Exception {
    if (isPostgresql(connection)) {
      // Use DO block with exception handling to protect against concurrent execution (ASTRUM-49154 pattern)
      runStatement(connection, """
          DO $$
          BEGIN
            BEGIN
              ALTER TABLE api_key_v2 ADD COLUMN created_by_user_id VARCHAR(200);
            EXCEPTION
              WHEN duplicate_column THEN NULL;
            END;
            BEGIN
              ALTER TABLE api_key_v2 ADD COLUMN created_by_realm VARCHAR(200);
            EXCEPTION
              WHEN duplicate_column THEN NULL;
            END;
          END $$;
          """);
    }
    else if (isH2(connection)) {
      // H2 supports IF NOT EXISTS natively
      if (!columnExists(connection, TABLE_NAME, "created_by_user_id")) {
        runStatement(connection,
            "ALTER TABLE " + TABLE_NAME + " ADD COLUMN IF NOT EXISTS created_by_user_id VARCHAR(200)");
      }
      if (!columnExists(connection, TABLE_NAME, "created_by_realm")) {
        runStatement(connection,
            "ALTER TABLE " + TABLE_NAME + " ADD COLUMN IF NOT EXISTS created_by_realm VARCHAR(200)");
      }
    }
  }
}
