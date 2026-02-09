
package org.globalart.astrum.repository.content.upgrades;

import java.sql.Connection;
import java.sql.SQLException;
import java.util.List;
import java.util.Optional;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.repository.Format;
import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;

import org.springframework.stereotype.Component;

/**
 * Adds H2-only indexes to {format}_asset_blob tables for existing databases.
 *
 * Related to ASTRUM-49154 - Migration created to allow removal of CREATE INDEX
 * from MyBatis createSchema() which could cause lock contention on startup.
 *
 * Note: PostgreSQL indexes are handled by AssetBlobAddedToRepositoryMigrationStep_2_27
 *
 * @since 3.87
 */
@Component
public class AssetBlobIndexesMigrationStep_2_67
    implements DatabaseMigrationStep
{
  private final List<Format> formats;

  @Autowired
  public AssetBlobIndexesMigrationStep_2_67(final List<Format> formats) {
    this.formats = formats;
  }

  @Override
  public Optional<String> version() {
    return Optional.of("2.67");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    // Only H2 needs these indexes created here
    // PostgreSQL indexes were created in AssetBlobAddedToRepositoryMigrationStep_2_27
    if (!isH2(connection)) {
      return;
    }

    for (Format format : formats) {
      String tableName = format.getValue() + "_asset_blob";
      if (tableExists(connection, tableName)) {
        createIndexes(connection, format.getValue(), tableName);
      }
    }
  }

  private void createIndexes(
      final Connection connection,
      final String format,
      final String tableName) throws SQLException
  {
    createIndexIfNotExists(connection, "idx_" + format + "_asset_blob_added_to_repository",
        tableName, "added_to_repository");

    createIndexIfNotExists(connection, "idx_" + format + "_asset_blob_created",
        tableName, "blob_created");
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

    runStatement(connection,
        String.format("CREATE INDEX IF NOT EXISTS %s ON %s (%s)", indexName, tableName, columns));
  }
}
