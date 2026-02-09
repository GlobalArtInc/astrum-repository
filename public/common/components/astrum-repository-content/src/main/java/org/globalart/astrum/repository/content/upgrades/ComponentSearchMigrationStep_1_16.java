
package org.globalart.astrum.repository.content.upgrades;

import java.sql.Connection;
import java.util.Optional;

import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;
import org.springframework.stereotype.Component;

/**
 * This migration step was deleted and is no longer available since the component_search table was deleted in a later
 * step
 */
@Component
public class ComponentSearchMigrationStep_1_16
    implements DatabaseMigrationStep
{

  @Override
  public Optional<String> version() {
    return Optional.of("1.16");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
  }
}
