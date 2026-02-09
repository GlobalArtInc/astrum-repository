
package org.globalart.astrum.repository.content.upgrades;

import java.sql.Connection;
import java.util.Optional;

import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Placeholder step, no-op, original step was moved to BrowseNodeMigrationStep_1_34 to retain proper order
 */
@Component
public class BrowseNodeMigrationStep_1_32
    implements DatabaseMigrationStep
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  @Override
  public Optional<String> version() {
    return Optional.of("1.32");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    // no-op simply placeholder so version doesn't get lost
  }
}
