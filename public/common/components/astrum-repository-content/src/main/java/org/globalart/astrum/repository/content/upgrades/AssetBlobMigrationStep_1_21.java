
package org.globalart.astrum.repository.content.upgrades;

import java.sql.Connection;
import java.sql.Statement;
import java.util.List;
import java.util.Optional;
import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.repository.Format;
import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;

import static java.util.Objects.requireNonNull;
import org.springframework.stereotype.Component;

/**
 * Drops index for blob_created on asset_blob table.
 */
@Component
public class AssetBlobMigrationStep_1_21
    implements DatabaseMigrationStep
{
  private static final String DROP_INDEX = "DROP INDEX IF EXISTS idx_%s_asset_blob_blob_created";

  private final List<Format> formats;

  @Autowired
  public AssetBlobMigrationStep_1_21(final List<Format> formats) {
    this.formats = requireNonNull(formats);
  }

  @Override
  public Optional<String> version() {
    return Optional.of("1.21");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    try (Statement st = connection.createStatement()) {
      for (Format format : formats) {
        st.execute(String.format(DROP_INDEX, format.getValue()));
      }
    }
  }
}
