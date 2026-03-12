
package org.globalart.astrum.upgrade.datastore.internal.steps;

import java.sql.Connection;
import java.util.Optional;

import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;

import org.springframework.stereotype.Component;

/**
 * Migration step for Node Heartbeat, activates the node-info table
 */
@Component
public class NodeHeartBeatMigrationStep_2_17_1
    implements DatabaseMigrationStep
{
  @Override
  public Optional<String> version() {
    return Optional.of("2.17.1");
  }

  @Override
  public void migrate(final Connection connection) {
    // This migration step is a no-op
  }
}
