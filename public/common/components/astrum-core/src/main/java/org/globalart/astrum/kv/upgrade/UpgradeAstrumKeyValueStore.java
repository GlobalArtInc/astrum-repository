
package org.globalart.astrum.kv.upgrade;

import java.sql.Connection;
import java.sql.PreparedStatement;
import java.sql.ResultSet;
import java.sql.SQLException;
import java.util.Collection;
import java.util.HashMap;
import java.util.Map;
import java.util.Optional;

import org.globalart.astrum.common.app.ManagedLifecycle;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.kv.AstrumKeyValue;
import org.globalart.astrum.kv.ValueType;
import org.globalart.astrum.upgrade.datastore.UpgradeConfigStoreSupport;

import com.fasterxml.jackson.databind.ObjectMapper;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.app.ManagedLifecycle.Phase.UPGRADE;

/**
 * UPGRADE-phase-safe equivalent of {@code GlobalKeyValueStore} that reads/writes the
 * {@code astrum_key_value} table directly via SQL instead of MyBatis + {@code ConfigStoreSupport}.
 *
 * <p>
 * {@code GlobalKeyValueStore} extends {@code ConfigStoreSupport}, which injects {@code EventManager}
 * (an EVENTS-phase component) and so cannot be injected into an UPGRADE-phase migration step. This
 * store performs the same persistence (a JSON object {@code {"value": <payload>}} in the JSON(B)
 * {@code value} column, with the {@link ValueType} name in {@code type}) without that dependency,
 * reusing the {@link AstrumKeyValue} data object.
 * </p>
 */
@Component
@ManagedLifecycle(phase = UPGRADE)
public class UpgradeAstrumKeyValueStore
    extends UpgradeConfigStoreSupport
{
  private static final String SELECT_H2 = "SELECT type, `value` FROM astrum_key_value WHERE `key` = ?";

  private static final String SELECT_PG = "SELECT type, value FROM astrum_key_value WHERE key = ?";

  private static final String MERGE_H2 =
      "MERGE INTO astrum_key_value (`key`, type, `value`, created) KEY(`key`) VALUES (?, ?, ?, now())";

  private static final String UPSERT_PG =
      "INSERT INTO astrum_key_value (key, type, value, created) VALUES (?, ?, ?, now()) "
          + "ON CONFLICT (key) DO UPDATE SET type = ?, value = ?, created = now()";

  private static final String DELETE_H2 = "DELETE FROM astrum_key_value WHERE `key` = ?";

  private static final String DELETE_PG = "DELETE FROM astrum_key_value WHERE key = ?";

  private final ObjectMapper mapper;

  @Autowired
  public UpgradeAstrumKeyValueStore(final DataSessionSupplier sessionSupplier, final ObjectMapper mapper) {
    super(sessionSupplier);
    this.mapper = checkNotNull(mapper);
  }

  /**
   * Gets a value by key.
   */
  public Optional<AstrumKeyValue> getKey(final String key) {
    try (Connection conn = openConnection()) {
      String sql = isH2(conn) ? SELECT_H2 : SELECT_PG;
      try (PreparedStatement statement = conn.prepareStatement(sql)) {
        statement.setString(1, key);
        try (ResultSet results = statement.executeQuery()) {
          if (!results.next()) {
            return Optional.empty();
          }
          AstrumKeyValue value = new AstrumKeyValue();
          value.setKey(key);
          value.setType(parseType(results.getString(1), key));
          value.setValue(readValueMap(results.getBytes(2)));
          return Optional.of(value);
        }
      }
    }
    catch (SQLException e) {
      throw new IllegalStateException("Failed to read key '" + key + "' from astrum_key_value", e);
    }
  }

  public <E> Optional<E> get(final String key, final Class<E> clazz) {
    return getKey(key).map(value -> value.getAsObject(mapper, clazz));
  }

  public Optional<Boolean> getBoolean(final String key) {
    return getKey(key).map(AstrumKeyValue::getAsBoolean);
  }

  public Optional<String> getString(final String key) {
    return getKey(key).map(AstrumKeyValue::getAsString);
  }

  public Optional<Integer> getInt(final String key) {
    return getKey(key).map(AstrumKeyValue::getAsInt);
  }

  public void setKey(final AstrumKeyValue keyValue) {
    byte[] json = writeValueMap(keyValue.value());
    try (Connection conn = openConnection()) {
      try {
        boolean h2 = isH2(conn);
        try (PreparedStatement statement = conn.prepareStatement(h2 ? MERGE_H2 : UPSERT_PG)) {
          statement.setString(1, keyValue.key());
          statement.setString(2, keyValue.type().name());
          setJsonParameter(statement, 3, json, h2);
          if (!h2) {
            // PostgreSQL upsert repeats type/value in the DO UPDATE clause
            statement.setString(4, keyValue.type().name());
            setJsonParameter(statement, 5, json, false);
          }
          statement.executeUpdate();
        }
        commitIfNeeded(conn);
      }
      catch (SQLException | RuntimeException e) {
        // Roll back on any failure (not just SQLException) so a RuntimeException mid-write does not leave
        // the in-flight transaction's disposition up to the pool. Precise rethrow keeps the checked type.
        rollbackIfNeeded(conn, e);
        throw e;
      }
    }
    catch (SQLException e) {
      throw new IllegalStateException("Failed to write key '" + keyValue.key() + "' to astrum_key_value", e);
    }
  }

  public void setBoolean(final String key, final boolean value) {
    setKey(new AstrumKeyValue(key, ValueType.BOOLEAN, value));
  }

  public void setInt(final String key, final int value) {
    setKey(new AstrumKeyValue(key, ValueType.NUMBER, value));
  }

  public void setString(final String key, final String value) {
    setKey(new AstrumKeyValue(key, ValueType.CHARACTER, value));
  }

  public void setObject(final String key, final Object value) {
    setKey(new AstrumKeyValue(key, ValueType.OBJECT, value));
  }

  /**
   * Removes a value by key, returning {@code true} if a record was deleted.
   */
  public boolean removeKey(final String key) {
    try (Connection conn = openConnection()) {
      try {
        String sql = isH2(conn) ? DELETE_H2 : DELETE_PG;
        boolean removed;
        try (PreparedStatement statement = conn.prepareStatement(sql)) {
          statement.setString(1, key);
          removed = statement.executeUpdate() > 0;
        }
        commitIfNeeded(conn);
        return removed;
      }
      catch (SQLException | RuntimeException e) {
        // Roll back on any failure (not just SQLException) so a RuntimeException mid-write does not leave
        // the in-flight transaction's disposition up to the pool. Precise rethrow keeps the checked type.
        rollbackIfNeeded(conn, e);
        throw e;
      }
    }
    catch (SQLException e) {
      throw new IllegalStateException("Failed to remove key '" + key + "' from astrum_key_value", e);
    }
  }

  /**
   * Removes multiple keys in a single batched transaction (one connection acquisition + one commit),
   * returning the total number of records deleted. Empty/blank keys are ignored. This avoids the
   * connection-per-key churn of calling {@link #removeKey(String)} in a loop, which is costly on a cold
   * pool during the UPGRADE hot path.
   */
  public int removeKeys(final Collection<String> keys) {
    if (keys == null || keys.isEmpty()) {
      return 0;
    }
    try (Connection conn = openConnection()) {
      try {
        String sql = isH2(conn) ? DELETE_H2 : DELETE_PG;
        int removed = 0;
        try (PreparedStatement statement = conn.prepareStatement(sql)) {
          for (String key : keys) {
            statement.setString(1, key);
            statement.addBatch();
          }
          for (int updateCount : statement.executeBatch()) {
            if (updateCount > 0) {
              removed += updateCount;
            }
          }
        }
        commitIfNeeded(conn);
        return removed;
      }
      catch (SQLException | RuntimeException e) {
        // Roll back on any failure (not just SQLException) so a RuntimeException mid-write does not leave
        // the in-flight transaction's disposition up to the pool. Precise rethrow keeps the checked type.
        rollbackIfNeeded(conn, e);
        throw e;
      }
    }
    catch (SQLException e) {
      throw new IllegalStateException("Failed to remove keys from astrum_key_value", e);
    }
  }

  /**
   * Resolves the stored {@code type} column to a {@link ValueType}, wrapping an unknown enum literal in an
   * {@link IllegalStateException} that names the offending value and key (rather than letting a bare
   * {@link IllegalArgumentException} from {@link ValueType#valueOf} escape without context).
   */
  private static ValueType parseType(final String typeName, final String key) {
    try {
      return ValueType.valueOf(typeName);
    }
    catch (IllegalArgumentException e) {
      throw new IllegalStateException(
          "Unknown ValueType '" + typeName + "' for key '" + key + "' in astrum_key_value", e);
    }
  }

  private Map<String, Object> readValueMap(final byte[] json) throws SQLException {
    if (json == null) {
      return new HashMap<>();
    }
    try {
      return mapper.readValue(json, new com.fasterxml.jackson.core.type.TypeReference<Map<String, Object>>()
      {
      });
    }
    catch (Exception e) {
      throw new SQLException("Failed to parse astrum_key_value JSON value", e);
    }
  }

  private byte[] writeValueMap(final Map<String, Object> value) {
    try {
      return mapper.writeValueAsBytes(value);
    }
    catch (Exception e) {
      throw new IllegalStateException("Failed to serialize astrum_key_value JSON value", e);
    }
  }
}
