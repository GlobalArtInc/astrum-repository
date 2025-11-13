
package org.globalart.astrum.internal.node.upgrade;

import java.sql.Connection;
import java.sql.PreparedStatement;
import java.sql.ResultSet;
import java.sql.SQLException;
import java.util.Optional;

import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.upgrade.datastore.UpgradeConfigStoreSupport;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Base for UPGRADE-phase-safe stores backed by a single-row, single-string-column table keyed on
 * {@code id = 1} (e.g. {@code node_id}, {@code deployment_id}). Reads/writes the column directly via SQL,
 * avoiding the {@code ConfigStoreSupport} -> {@code EventManager} (EVENTS phase) dependency.
 *
 * <p>
 * The {@code table} and {@code column} names are compile-time constants supplied by subclasses (never user
 * input), so composing the SQL from them is not an injection vector.
 * </p>
 */
public abstract class SingleRowStringUpgradeStore
    extends UpgradeConfigStoreSupport
{
  private final String table;

  private final String selectSql;

  private final String mergeH2Sql;

  private final String upsertPgSql;

  protected SingleRowStringUpgradeStore(
      final DataSessionSupplier sessionSupplier,
      final String table,
      final String column)
  {
    super(sessionSupplier);
    this.table = checkNotNull(table);
    checkNotNull(column);
    this.selectSql = String.format("SELECT %s FROM %s WHERE id = 1", column, table);
    this.mergeH2Sql = String.format("MERGE INTO %s (id, %s) VALUES (1, ?)", table, column);
    this.upsertPgSql = String.format(
        "INSERT INTO %s (id, %s) VALUES (1, ?) ON CONFLICT (id) DO UPDATE SET %s = ?", table, column, column);
  }

  /**
   * Reads the single stored value, or empty if the row is absent.
   */
  protected Optional<String> read() {
    try (Connection conn = openConnection();
        PreparedStatement statement = conn.prepareStatement(selectSql);
        ResultSet results = statement.executeQuery()) {
      return results.next() ? Optional.ofNullable(results.getString(1)) : Optional.empty();
    }
    catch (SQLException e) {
      throw new IllegalStateException("Failed to read " + table, e);
    }
  }

  /**
   * Upserts the single stored value (id = 1).
   */
  protected void write(final String value) {
    checkNotNull(value, "value");
    try (Connection conn = openConnection()) {
      try {
        boolean h2 = isH2(conn);
        try (PreparedStatement statement = conn.prepareStatement(h2 ? mergeH2Sql : upsertPgSql)) {
          statement.setString(1, value);
          if (!h2) {
            statement.setString(2, value);
          }
          statement.executeUpdate();
        }
        commitIfNeeded(conn);
      }
      catch (SQLException | RuntimeException e) {
        // Roll back on any failure so a dirty transaction is not returned to the pool: try-with-resources
        // calls close(), not rollback(), and pool disposition of an in-flight transaction is unsafe.
        rollbackIfNeeded(conn, e);
        throw e;
      }
    }
    catch (SQLException e) {
      throw new IllegalStateException("Failed to write " + table, e);
    }
  }
}
