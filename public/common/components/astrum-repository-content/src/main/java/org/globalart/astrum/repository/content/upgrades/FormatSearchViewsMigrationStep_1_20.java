
package org.globalart.astrum.repository.content.upgrades;

import java.sql.Connection;
import java.sql.Statement;
import java.util.List;
import java.util.Optional;
import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.repository.Format;
import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;
import org.springframework.stereotype.Component;

/**
 * Drop legacy format search views.
 */
@Component
public class FormatSearchViewsMigrationStep_1_20
    implements DatabaseMigrationStep
{
  private static final String DROP_FORMAT_SEARCH_VIEW = "DROP VIEW IF EXISTS %s_component_search CASCADE";

  private final List<Format> formats;

  @Autowired
  public FormatSearchViewsMigrationStep_1_20(final List<Format> formats) {
    this.formats = formats;
  }

  @Override
  public Optional<String> version() {
    return Optional.of("1.20");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    try (Statement st = connection.createStatement()) {
      for (Format format : formats) {
        st.execute(String.format(DROP_FORMAT_SEARCH_VIEW, format.getValue()));
      }
    }
  }
}
