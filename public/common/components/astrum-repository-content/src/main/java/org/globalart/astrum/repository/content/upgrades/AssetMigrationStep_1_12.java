
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
 * Creates index for last_updated on each asset table.
 */
@Component
public class AssetMigrationStep_1_12
    implements DatabaseMigrationStep
{
  private static final String CREATE_INDEX =
      "CREATE INDEX IF NOT EXISTS idx_%s_asset_last_updated ON %s_asset (last_updated)";

  private final List<Format> formats;

  @Autowired
  public AssetMigrationStep_1_12(final List<Format> formats) {
    this.formats = requireNonNull(formats);
  }

  @Override
  public Optional<String> version() {
    return Optional.of("1.12");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    try (Statement st = connection.createStatement()) {
      for (Format format : formats) {
        st.execute(String.format(CREATE_INDEX, format.getValue(), format.getValue()));
      }
    }
  }
}
