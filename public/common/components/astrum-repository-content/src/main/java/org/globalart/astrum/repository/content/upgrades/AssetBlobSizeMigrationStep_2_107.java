
package org.globalart.astrum.repository.content.upgrades;

import java.sql.Connection;
import java.sql.SQLException;
import java.util.List;
import java.util.Optional;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.repository.Format;
import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;

import static java.util.Objects.requireNonNull;
import org.springframework.stereotype.Component;

/**
 * Adds the asset_blob_size column to {format}_asset tables.
 *
 * This column caches the blob size from the asset_blob table for performance,
 * avoiding joins when querying asset sizes.
 *
 * Related to ASTRUM-50612 - Migration created to add missing column for customers
 * upgrading directly from pre-3.67.0 to 3.87.0+. The column was originally added
 * via ALTER TABLE in 3.67.0, but ASTRUM-49154 removed ALTER TABLE statements to
 * fix lock contention. This migration ensures all upgrade paths receive the column.
 */
@Component
public class AssetBlobSizeMigrationStep_2_107
    implements DatabaseMigrationStep
{
  private final List<Format> formats;

  @Autowired
  public AssetBlobSizeMigrationStep_2_107(final List<Format> formats) {
    this.formats = requireNonNull(formats);
  }

  @Override
  public Optional<String> version() {
    return Optional.of("2.107");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    for (Format format : formats) {
      String tableName = format.getValue() + "_asset";

      if (tableExists(connection, tableName)) {
        addColumnIfNotExists(connection, tableName);
      }
    }
  }

  private void addColumnIfNotExists(final Connection connection, final String tableName) throws SQLException {
    if (columnExists(connection, tableName, "asset_blob_size")) {
      return; // Column already exists
    }

    if (isPostgresql(connection)) {
      // Use exception handling to avoid lock contention (ASTRUM-49154)
      runStatement(connection, String.format("""
          DO $$
          BEGIN
            BEGIN
              ALTER TABLE %s
              ADD COLUMN asset_blob_size BIGINT;
            EXCEPTION
              WHEN duplicate_column THEN NULL;
            END;
          END $$;
          """, tableName));
    }
    else if (isH2(connection)) {
      // H2 can safely use IF NOT EXISTS
      runStatement(connection, String.format(
          "ALTER TABLE %s ADD COLUMN IF NOT EXISTS asset_blob_size BIGINT",
          tableName));
    }
  }
}
