
package org.globalart.astrum.content.maven.upgrades;

import java.sql.Connection;
import java.sql.SQLException;
import java.util.Optional;

import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;

import org.springframework.stereotype.Component;

/**
 * Adds the base_version column to maven2_component table.
 *
 * This column stores the base version (without SNAPSHOT timestamp) for Maven
 * components, used by cleanup policies to retain N snapshots.
 *
 * Related to ASTRUM-49154 - Migration created to allow removal of ALTER TABLE
 * from MyBatis extendSchema() which was causing lock contention on startup.
 *
 * @since 3.87
 */
@Component
public class Maven2ComponentBaseVersionMigrationStep_2_29
    implements DatabaseMigrationStep
{
  private static final String TABLE_NAME = "maven2_component";

  private static final String COLUMN_NAME = "base_version";

  private static final String INDEX_NAME = "idx_maven2_component_base_version";

  @Override
  public Optional<String> version() {
    return Optional.of("2.29");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    if (tableExists(connection, TABLE_NAME)) {
      addColumnIfNotExists(connection);
      addIndexIfNotExists(connection);
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
              ALTER TABLE maven2_component ADD COLUMN base_version VARCHAR NULL;
            EXCEPTION
              WHEN duplicate_column THEN NULL;
            END;
          END $$;
          """);
    }
    else if (isH2(connection)) {
      // H2 can safely use IF NOT EXISTS
      runStatement(connection,
          "ALTER TABLE maven2_component ADD COLUMN IF NOT EXISTS base_version VARCHAR NULL");
    }
  }

  private void addIndexIfNotExists(final Connection connection) throws SQLException {
    if (!indexExists(connection, INDEX_NAME)) {
      runStatement(connection, String.format(
          "CREATE INDEX IF NOT EXISTS %s ON %s (%s)",
          INDEX_NAME, TABLE_NAME, COLUMN_NAME));
    }
  }
}
