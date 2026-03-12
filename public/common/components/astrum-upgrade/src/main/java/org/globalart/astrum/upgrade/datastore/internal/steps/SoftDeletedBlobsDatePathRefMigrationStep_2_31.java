
package org.globalart.astrum.upgrade.datastore.internal.steps;

import java.sql.Connection;
import java.sql.SQLException;
import java.util.Optional;

import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;

import org.springframework.stereotype.Component;

/**
 * Adds the date_path_ref column to soft_deleted_blobs table.
 *
 * This column stores a reference date used to generate the date-based path
 * structure for soft-deleted blobs in the blob store.
 *
 * Related to ASTRUM-49154 - Migration created to allow removal of ALTER TABLE
 * from MyBatis createSchema() which was causing lock contention on startup.
 *
 * @since 3.87
 */
@Component
public class SoftDeletedBlobsDatePathRefMigrationStep_2_31
    implements DatabaseMigrationStep
{
  private static final String TABLE_NAME = "soft_deleted_blobs";

  private static final String COLUMN_NAME = "date_path_ref";

  @Override
  public Optional<String> version() {
    return Optional.of("2.31");
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
              ALTER TABLE soft_deleted_blobs ADD COLUMN date_path_ref TIMESTAMP WITH TIME ZONE;
            EXCEPTION
              WHEN duplicate_column THEN NULL;
            END;
          END $$;
          """);
    }
    else if (isH2(connection)) {
      // H2 can safely use IF NOT EXISTS
      runStatement(connection,
          "ALTER TABLE soft_deleted_blobs ADD COLUMN IF NOT EXISTS date_path_ref TIMESTAMP WITH TIME ZONE");
    }
  }
}
