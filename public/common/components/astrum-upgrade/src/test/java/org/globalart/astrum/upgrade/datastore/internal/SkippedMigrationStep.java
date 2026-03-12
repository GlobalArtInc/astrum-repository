
package org.globalart.astrum.upgrade.datastore.internal;

import java.sql.Connection;
import java.sql.Statement;
import java.util.Optional;

import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Skipped test migration step
 *
 * @since 3.41
 */
public class SkippedMigrationStep
    implements DatabaseMigrationStep
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  @Override
  public Optional<String> version() {
    return Optional.of("2.0");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    try (Statement stmt = connection.createStatement()) {
      stmt.execute("CREATE TABLE IF NOT EXISTS skipped ( name VARCHAR(50) )");
      log.info("Created example table.");
      stmt.execute("INSERT INTO skipped (name) values('fawkes')");
      log.info("Inserted name = 'fawkes' into example table.");
    }
  }
}
