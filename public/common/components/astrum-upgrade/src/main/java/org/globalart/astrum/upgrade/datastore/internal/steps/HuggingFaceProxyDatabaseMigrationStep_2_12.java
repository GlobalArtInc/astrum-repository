
package org.globalart.astrum.upgrade.datastore.internal.steps;

import java.sql.Connection;
import java.util.Optional;
import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;
import org.springframework.stereotype.Component;

@Component
public class HuggingFaceProxyDatabaseMigrationStep_2_12
    implements DatabaseMigrationStep
{
  @Override
  public Optional<String> version() {
    return Optional.of("2.12");
  }

  @Override
  public void migrate(final Connection connection) {
    // No-op, this makes the HuggingFace proxy repositories available
  }
}
