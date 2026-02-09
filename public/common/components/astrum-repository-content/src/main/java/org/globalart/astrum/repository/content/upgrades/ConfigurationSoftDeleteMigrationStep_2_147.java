
package org.globalart.astrum.repository.content.upgrades;

import java.sql.Connection;
import java.util.Optional;

import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;
import org.springframework.stereotype.Component;

/**
 * This is a placeholder for a removed migration step.
 *
 * The original step was introduced by the Logical Deletion Recovery initiative (ASTRUM-51733) and shipped
 * to dev/staging tenants, recording a {@code flyway_schema_history} row. After the feature was reverted,
 * the step had to remain present as a no-op so Flyway can still match the recorded row to a Java migration
 * and does not treat the schema as being from a later version of Astrum Repository.
 */
@Component
public class ConfigurationSoftDeleteMigrationStep_2_147
    implements DatabaseMigrationStep
{
  @Override
  public Optional<String> version() {
    return Optional.of("2.147");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    // noop
  }
}
