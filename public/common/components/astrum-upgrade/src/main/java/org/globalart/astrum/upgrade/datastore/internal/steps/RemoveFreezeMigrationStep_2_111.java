
package org.globalart.astrum.upgrade.datastore.internal.steps;

import java.sql.Connection;
import java.util.Optional;
import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Removes the freeze_requests table as the freeze mechanism is no longer supported.
 */
@Component
public class RemoveFreezeMigrationStep_2_111
    implements DatabaseMigrationStep
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private static final String DROP_FREEZE_REQUESTS_TABLE = """
      DROP TABLE IF EXISTS freeze_requests;
      """;

  @Override
  public Optional<String> version() {
    return Optional.of("2.111");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    if (tableExists(connection, "freeze_requests")) {
      log.info("Dropping freeze_requests table as freeze mechanism is no longer supported");
      runStatement(connection, DROP_FREEZE_REQUESTS_TABLE);
    }
  }
}
