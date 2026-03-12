
package org.globalart.astrum.upgrade.datastore;

import java.sql.Connection;
import java.sql.PreparedStatement;
import java.sql.SQLException;

import org.globalart.astrum.common.stateguard.StateGuardLifecycleSupport;
import org.globalart.astrum.datastore.api.DataSessionSupplier;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;

/**
 * Base class for lifecycle-safe "Upgrade" stores used by UPGRADE-phase migration steps.
 *
 * <p>
 * These stores mirror their {@code ConfigStoreSupport} counterpart but perform direct SQL through a raw
 * JDBC {@link Connection} and, crucially, do <em>not</em> inject {@code EventManager}.
 * {@code ConfigStoreSupport} {@code @Autowired}s {@code EventManager} ({@code EventManagerImpl} is
 * {@code @ManagedLifecycle(phase = EVENTS)}); EVENTS starts after the UPGRADE phase, so a
 * {@code ConfigStoreSupport}-based store cannot be safely injected into an UPGRADE-phase migration step
 * without forcing premature initialization of an EVENTS-phase component.
 * </p>
 *
 * <p>
 * Subclasses should be annotated {@code @Component} and {@code @ManagedLifecycle(phase = UPGRADE)} and
 * live in an {@code upgrade} package alongside the {@code ConfigStoreSupport} store they replace, so they
 * can reuse its data objects. Data is accessed against the default datastore, which is available from the
 * STORAGE phase onward.
 * </p>
 */
public abstract class UpgradeConfigStoreSupport
    extends StateGuardLifecycleSupport
{
  protected final DataSessionSupplier sessionSupplier;

  protected UpgradeConfigStoreSupport(final DataSessionSupplier sessionSupplier) {
    this.sessionSupplier = checkNotNull(sessionSupplier);
  }

  /**
   * Opens a new JDBC connection to the default datastore. Callers are responsible for closing it
   * (use try-with-resources).
   */
  protected Connection openConnection() throws SQLException {
    return sessionSupplier.openConnection(DEFAULT_DATASTORE_NAME);
  }

  protected boolean isH2(final Connection conn) throws SQLException {
    return "H2".equals(conn.getMetaData().getDatabaseProductName());
  }

  protected boolean isPostgresql(final Connection conn) throws SQLException {
    return "PostgreSQL".equals(conn.getMetaData().getDatabaseProductName());
  }

  /**
   * Binds a JSON(B) value to a statement parameter, matching {@code AbstractJsonTypeHandler}: H2 expects
   * the JSON as a UTF-8 {@code byte[]}, PostgreSQL as a UTF-8 {@code String}. Delegates to the shared
   * {@link JsonParameterBinder} (the single source of truth also used by
   * {@code DatabaseMigrationStep#setJsonParameter}) so the two dialect bindings cannot drift.
   */
  protected void setJsonParameter(
      final PreparedStatement statement,
      final int index,
      final byte[] json,
      final boolean h2) throws SQLException
  {
    JsonParameterBinder.setJsonParameter(statement, index, json, h2);
  }

  /**
   * Commits the connection if it is not in auto-commit mode (raw pooled connections may differ).
   */
  protected void commitIfNeeded(final Connection conn) throws SQLException {
    if (!conn.getAutoCommit()) {
      conn.commit();
    }
  }

  /**
   * Rolls back the connection if it is not in auto-commit mode, suppressing any rollback failure onto the
   * original {@code cause} (a {@link SQLException} or {@link RuntimeException} from the failed write). Call
   * this when a write fails so the in-flight transaction is explicitly abandoned rather than relying on the
   * connection pool's (pool-specific) disposition of a dirty connection on return.
   */
  protected void rollbackIfNeeded(final Connection conn, final Exception cause) {
    try {
      if (!conn.getAutoCommit()) {
        conn.rollback();
      }
    }
    catch (SQLException rollbackFailure) {
      cause.addSuppressed(rollbackFailure);
    }
  }
}
