
package org.globalart.astrum.self.hosted.internal.capability.proxy;

import java.sql.Connection;
import java.util.Optional;

import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;

import org.springframework.stereotype.Component;

@Component
public class ForwardedRequestCapabilityUpgradeStep_2_131
    implements DatabaseMigrationStep
{
  static final String VERSION = "2.131";

  @Override
  public Optional<String> version() {
    return Optional.of(VERSION);
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    // no-op
  }
}
