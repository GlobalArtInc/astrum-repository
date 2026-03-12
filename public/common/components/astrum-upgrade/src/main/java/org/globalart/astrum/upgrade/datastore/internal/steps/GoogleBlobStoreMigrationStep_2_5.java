
package org.globalart.astrum.upgrade.datastore.internal.steps;

import java.sql.Connection;
import java.util.Optional;
import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;
import org.springframework.stereotype.Component;

@Component
public class GoogleBlobStoreMigrationStep_2_5
    implements DatabaseMigrationStep
{
  @Override
  public Optional<String> version() {
    return Optional.of("2.5");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    // No-op, this makes the GoogleBlobStore available
  }
}
