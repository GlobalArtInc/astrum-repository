
package org.globalart.astrum.internal.node.datastore;

import java.sql.Connection;
import java.util.Optional;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Originally migrated the legacy on disk node identifier to the database where it may be used to identify a single
 * node, or a cluster. Now a no-op {@see NodeIdInitializerimpl}
 */
@Component
public class NodeIdUpgradeStep_1_14
    implements DatabaseMigrationStep
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  @Autowired
  public NodeIdUpgradeStep_1_14() {
  }

  @Override
  public Optional<String> version() {
    return Optional.of("1.14");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    // no-op
  }
}
