
package org.globalart.astrum.audit.internal.store;

import java.sql.Connection;
import java.util.Optional;

import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;

import org.springframework.stereotype.Component;

/**
 * Creates audit_events table for queryable audit event storage.
 */
@Component
public class AuditEventMigrationStep_2_115
    implements DatabaseMigrationStep
{
  private static final String TABLE_NAME = "audit_events";

  @Override
  public Optional<String> version() {
    return Optional.of("2.115");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    if (!tableExists(connection, TABLE_NAME)) {
      return;
    }

    if (isPostgresql(connection)) {
      runStatement(connection, """
          CREATE INDEX IF NOT EXISTS idx_audit_events_domain_timestamp
            ON audit_events (domain, timestamp DESC)
          """);

      runStatement(connection, """
          CREATE INDEX IF NOT EXISTS idx_audit_events_type
            ON audit_events (type)
          """);

      runStatement(connection, """
          CREATE INDEX IF NOT EXISTS idx_audit_events_initiator
            ON audit_events (initiator)
          """);
    }
    else {
      runStatement(connection, """
          CREATE INDEX IF NOT EXISTS idx_audit_events_domain_timestamp
            ON audit_events (domain, timestamp DESC)
          """);

      runStatement(connection, """
          CREATE INDEX IF NOT EXISTS idx_audit_events_type
            ON audit_events (type)
          """);

      runStatement(connection, """
          CREATE INDEX IF NOT EXISTS idx_audit_events_initiator
            ON audit_events (initiator)
          """);
    }
  }
}
