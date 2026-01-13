
package org.globalart.astrum.audit.internal.store;

import java.sql.Connection;
import java.util.Optional;

import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;

import org.springframework.stereotype.Component;

/**
 * Adds {@code idx_audit_events_context} for repositories filtering change history by context (repository name).
 */
@Component
public class AuditEventsContextIndexMigrationStep_2_116
    implements DatabaseMigrationStep
{
  private static final String TABLE_NAME = "audit_events";

  private static final String INDEX_NAME = "idx_audit_events_context";

  @Override
  public Optional<String> version() {
    return Optional.of("2.116");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    if (!tableExists(connection, TABLE_NAME)) {
      return;
    }
    runStatement(connection, String.format(
        "CREATE INDEX IF NOT EXISTS %s ON %s (context)", INDEX_NAME, TABLE_NAME));
  }
}
