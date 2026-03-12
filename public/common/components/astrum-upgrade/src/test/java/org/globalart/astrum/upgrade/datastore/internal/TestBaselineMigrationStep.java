
package org.globalart.astrum.upgrade.datastore.internal;

import java.sql.Connection;
import java.util.Optional;

import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class TestBaselineMigrationStep
    implements DatabaseMigrationStep
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  @Override
  public Optional<String> version() {
    return Optional.of("1.8");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    // nothing to do here
  }
}
