
package org.globalart.astrum.upgrade.datastore.internal.steps;

import java.sql.Connection;
import java.util.Optional;

import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;
import org.springframework.stereotype.Component;

/**
 * This is a placeholder for a removed migration step
 */
@Component
public class NodeHeartbeatUpgrade_1_24
    implements DatabaseMigrationStep
{
  @Override
  public Optional<String> version() {
    return Optional.of("1.24");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    // noop
  }
}
