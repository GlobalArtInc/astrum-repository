
package org.globalart.astrum.upgrade.datastore;

import java.nio.charset.StandardCharsets;
import java.sql.PreparedStatement;
import java.sql.SQLException;

/**
 * Single source of truth for binding a JSON(B) value to a prepared-statement parameter using the
 * dialect-correct type: H2 stores JSON(B) as a UTF-8 {@code byte[]}, PostgreSQL as a UTF-8 {@code String}
 * (matching {@code AbstractJsonTypeHandler}).
 *
 * <p>
 * Shared by {@link DatabaseMigrationStep#setJsonParameter} (for direct-SQL migration steps) and
 * {@link UpgradeConfigStoreSupport#setJsonParameter} (for the upgrade-phase stores) so the two cannot drift:
 * a wrong-dialect binding silently corrupts the stored JSON, so this must have exactly one implementation.
 * </p>
 */
final class JsonParameterBinder
{
  private JsonParameterBinder() {
    // static utility
  }

  static void setJsonParameter(
      final PreparedStatement statement,
      final int index,
      final byte[] json,
      final boolean h2) throws SQLException
  {
    if (h2) {
      statement.setBytes(index, json);
    }
    else {
      statement.setString(index, new String(json, StandardCharsets.UTF_8));
    }
  }
}
