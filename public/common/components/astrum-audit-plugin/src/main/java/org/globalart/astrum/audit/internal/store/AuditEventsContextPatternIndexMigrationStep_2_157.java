
package org.globalart.astrum.audit.internal.store;

import java.sql.Connection;
import java.util.Optional;

import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;

import org.springframework.stereotype.Component;

/**
 * Replaces the plain-btree {@code idx_audit_events_context} with a text_pattern_ops btree so
 * {@code context LIKE 'X:%'} in the audit-log query is index-usable regardless of the database
 * collation. On PostgreSQL under non-C collations (e.g. {@code en_US.utf8}) a plain btree cannot
 * serve {@code LIKE 'prefix%'}; text_pattern_ops uses byte-wise ordering, so it can. H2's default
 * text ordering is already codepoint-based, so the H2 branch is a no-op.
 */
@Component
public class AuditEventsContextPatternIndexMigrationStep_2_157
    implements DatabaseMigrationStep
{
  private static final String TABLE_NAME = "audit_events";

  private static final String OLD_INDEX_NAME = "idx_audit_events_context";

  private static final String NEW_INDEX_NAME = "idx_audit_events_context_pattern";

  @Override
  public Optional<String> version() {
    return Optional.of("2.157");
  }

  @Override
  public boolean canExecuteInTransaction() {
    return false;
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    if (!tableExists(connection, TABLE_NAME)) {
      return;
    }
    if (isPostgresql(connection)) {
      runStatement(connection, String.format(
          "CREATE INDEX CONCURRENTLY IF NOT EXISTS %s ON %s (context text_pattern_ops)",
          NEW_INDEX_NAME, TABLE_NAME));
      runStatement(connection, String.format(
          "DROP INDEX CONCURRENTLY IF EXISTS %s", OLD_INDEX_NAME));
    }
  }
}
