
package org.globalart.astrum.repository.content.search.capability;

import java.sql.Connection;
import java.util.Optional;

import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;
import org.springframework.stereotype.Component;

@Component
public class SearchCapabilityStep_2_1
    implements DatabaseMigrationStep
{
  @Override
  public Optional<String> version() {
    return Optional.of("2.1");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    // No-op, this makes the SearchCapability available
  }
}
