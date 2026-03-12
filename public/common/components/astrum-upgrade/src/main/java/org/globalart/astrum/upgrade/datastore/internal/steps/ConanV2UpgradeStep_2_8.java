
package org.globalart.astrum.upgrade.datastore.internal.steps;

import java.sql.Connection;
import java.util.Optional;
import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;
import org.springframework.stereotype.Component;

@Component
public class ConanV2UpgradeStep_2_8
    implements DatabaseMigrationStep
{
  @Override
  public Optional<String> version() {
    return Optional.of("2.8");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    // No-op, this makes conan V2 proxy/group repositories available
  }
}
