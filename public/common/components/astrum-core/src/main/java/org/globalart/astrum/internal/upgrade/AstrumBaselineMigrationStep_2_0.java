
package org.globalart.astrum.internal.upgrade;

import java.sql.Connection;
import java.util.Optional;

import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * No-op baseline version for Zero Downtime Upgrades
 */
@Component
public class AstrumBaselineMigrationStep_2_0
    implements DatabaseMigrationStep
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  @Override
  public Optional<String> version() {
    return Optional.of("2.0");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    // no-op
  }
}
