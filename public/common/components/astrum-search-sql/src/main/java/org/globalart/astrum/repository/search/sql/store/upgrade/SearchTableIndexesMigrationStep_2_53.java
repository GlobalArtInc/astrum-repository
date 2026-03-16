
package org.globalart.astrum.repository.search.sql.store.upgrade;

import java.sql.Connection;
import java.util.Optional;

import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;

import org.springframework.stereotype.Component;

/**
 * No-op migration step - Original functionality moved to SearchTableIndexesMigrationStep_2_77.
 *
 * @since 3.87
 */
@Component
public class SearchTableIndexesMigrationStep_2_53
    implements DatabaseMigrationStep
{
  @Override
  public Optional<String> version() {
    return Optional.of("2.53");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    // No-op - functionality moved to SearchTableIndexesMigrationStep_2_77
  }
}
