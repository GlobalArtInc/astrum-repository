
package org.globalart.astrum.repository.content.upgrades;

import java.sql.Connection;
import java.sql.Statement;
import java.util.Optional;

import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;
import org.springframework.stereotype.Component;

/**
 * Deletes the component_search table.
 */
@Component
public class ComponentSearchMigrationStep_1_22
    implements DatabaseMigrationStep
{
  private static final String DROP_COMPONENT_SEARCH = "DROP TABLE IF EXISTS component_search";

  @Override
  public Optional<String> version() {
    return Optional.of("1.22");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    try (Statement st = connection.createStatement()) {
      st.execute(DROP_COMPONENT_SEARCH);
    }
  }
}
