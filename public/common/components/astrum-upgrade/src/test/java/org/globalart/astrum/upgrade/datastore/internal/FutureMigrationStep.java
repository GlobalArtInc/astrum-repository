
package org.globalart.astrum.upgrade.datastore.internal;

import java.sql.Connection;
import java.util.Optional;

import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Test migration step from future
 * 
 * @since 3.36
 */
public class FutureMigrationStep
    implements DatabaseMigrationStep
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  @Override
  public Optional<String> version() {
    return Optional.of("4.5.6");
  }

  @Override
  public void migrate(final Connection connection) {
    // . . .
  }
}
