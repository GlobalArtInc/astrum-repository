
package org.globalart.astrum.upgrade.datastore.internal.steps;

import java.sql.Connection;
import java.util.Optional;

import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Delete all tokens from the {@code distributed_auth_ticket_cache} table
 */
@Component
public class DistributedAuthTicketMigrationStep_1_30
    implements DatabaseMigrationStep
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private static final String DELETE_ALL_RECORDS = "TRUNCATE TABLE distributed_auth_ticket_cache";

  @Override
  public Optional<String> version() {
    return Optional.of("1.30");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    if (tableExists(connection, "distributed_auth_ticket_cache")) {
      log.info("Deleting all tokens form the distributed_auth_ticket_cache table");
      runStatement(connection, DELETE_ALL_RECORDS);
    }
  }
}
