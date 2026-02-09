
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
 * Adds the external_metadata column to {format}_asset_blob tables.
 *
 * This column stores external metadata such as ETag and last modified date
 * for assets stored in external blob stores (e.g., S3).
 *
 * @since 3.88
 */
@Component
public class AssetBlobExternalMetadataMigrationStep_2_70
    implements DatabaseMigrationStep
{
  private final List<Format> formats;

  @Autowired
  public AssetBlobExternalMetadataMigrationStep_2_70(final List<Format> formats) {
    this.formats = requireNonNull(formats);
  }

  @Override
  public Optional<String> version() {
    return Optional.of("2.70");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    for (Format format : formats) {
      String tableName = format.getValue() + "_asset_blob";

      if (tableExists(connection, tableName)) {
        addColumnIfNotExists(connection, tableName);
      }
    }
  }

  private void addColumnIfNotExists(final Connection connection, final String tableName) throws SQLException {
    if (columnExists(connection, tableName, "external_metadata")) {
      return;
    }

    if (isPostgresql(connection)) {
      runStatement(connection, String.format("""
          DO $$
          BEGIN
            BEGIN
              ALTER TABLE %s
              ADD COLUMN external_metadata JSONB;
            EXCEPTION
              WHEN duplicate_column THEN NULL;
            END;
          END $$;
          """, tableName));
    }
    else if (isH2(connection)) {
      runStatement(connection, String.format(
          "ALTER TABLE %s ADD COLUMN IF NOT EXISTS external_metadata VARCHAR",
          tableName));
    }
  }
}
