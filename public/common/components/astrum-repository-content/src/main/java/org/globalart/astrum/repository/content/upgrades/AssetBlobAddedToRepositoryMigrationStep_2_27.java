
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
 * Adds the added_to_repository column and indexes to {format}_asset_blob tables.
 *
 * This column tracks when asset blobs are added to the repository, used for
 * reconciliation and auditing purposes.
 *
 * Also adds indexes on added_to_repository and blob_created columns for performance.
 *
 * Related to ASTRUM-49154 - Migration created to allow removal of ALTER TABLE
 * and CREATE INDEX from MyBatis createSchema() which was causing lock contention on startup.
 *
 * @since 3.87
 */
@Component
public class AssetBlobAddedToRepositoryMigrationStep_2_27
    implements DatabaseMigrationStep
{
  private final List<Format> formats;

  @Autowired
  public AssetBlobAddedToRepositoryMigrationStep_2_27(final List<Format> formats) {
    this.formats = requireNonNull(formats);
  }

  @Override
  public Optional<String> version() {
    return Optional.of("2.27");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    for (Format format : formats) {
      String tableName = format.getValue() + "_asset_blob";

      if (tableExists(connection, tableName)) {
        addColumnIfNotExists(connection, tableName);
        addAddedToRepositoryIndexIfNotExists(connection, tableName, format.getValue());
        addBlobCreatedIndexIfNotExists(connection, tableName, format.getValue());
      }
    }
  }

  private void addColumnIfNotExists(final Connection connection, final String tableName) throws SQLException {
    if (columnExists(connection, tableName, "added_to_repository")) {
      return; // Column already exists
    }

    if (isPostgresql(connection)) {
      // Use exception handling to avoid lock contention (ASTRUM-49154)
      runStatement(connection, String.format("""
          DO $$
          BEGIN
            BEGIN
              ALTER TABLE %s
              ADD COLUMN added_to_repository TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP;
            EXCEPTION
              WHEN duplicate_column THEN NULL;
            END;
          END $$;
          """, tableName));
    }
    else if (isH2(connection)) {
      // H2 can safely use IF NOT EXISTS
      runStatement(connection, String.format(
          "ALTER TABLE %s ADD COLUMN IF NOT EXISTS added_to_repository TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP",
          tableName));
    }
  }

  private void addAddedToRepositoryIndexIfNotExists(
      final Connection connection,
      final String tableName,
      final String format) throws SQLException
  {
    String indexName = "idx_" + format + "_asset_blob_added_to_repository";

    if (!indexExists(connection, indexName)) {
      runStatement(connection, String.format(
          "CREATE INDEX IF NOT EXISTS %s ON %s (added_to_repository)",
          indexName, tableName));
    }
  }

  private void addBlobCreatedIndexIfNotExists(
      final Connection connection,
      final String tableName,
      final String format) throws SQLException
  {
    String indexName = "idx_" + format + "_asset_blob_created";

    if (!indexExists(connection, indexName)) {
      runStatement(connection, String.format(
          "CREATE INDEX IF NOT EXISTS %s ON %s (blob_created)",
          indexName, tableName));
    }
  }
}
