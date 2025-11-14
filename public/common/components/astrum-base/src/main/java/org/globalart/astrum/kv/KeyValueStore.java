
package org.globalart.astrum.kv;

import java.util.Optional;

/**
 * Key-value store interface for storing and retrieving configuration data.
 */
public interface KeyValueStore
{
  /**
   * Gets a value by the given key.
   *
   * @param key a string key
   * @return {@link Optional} of {@link AstrumKeyValue}
   */
  Optional<AstrumKeyValue> getKey(String key);

  /**
   * Gets a value by the given key and deserializes it to the specified type.
   *
   * @param key the string key
   * @param clazz the class to deserialize to
   * @param <E> the type parameter
   * @return {@link Optional} of the deserialized object
   */
  <E> Optional<E> get(String key, Class<E> clazz);

  /**
   * Gets a boolean value by the given key.
   *
   * @param key the string key
   * @return {@link Optional} of {@link Boolean}
   */
  Optional<Boolean> getBoolean(String key);

  /**
   * Gets a string value by the given key.
   *
   * @param key the string key
   * @return {@link Optional} of {@link String}
   */
  Optional<String> getString(String key);

  /**
   * Gets an integer value by the given key.
   *
   * @param key the string key
   * @return {@link Optional} of {@link Integer}
   */
  Optional<Integer> getInt(String key);

  /**
   * Sets a key-value record.
   *
   * @param keyValue record to be created/updated
   */
  void setKey(AstrumKeyValue keyValue);

  /**
   * Sets a boolean value for the given key.
   *
   * @param key the string key
   * @param value the boolean value
   */
  void setBoolean(String key, boolean value);

  /**
   * Sets an integer value for the given key.
   *
   * @param key the string key
   * @param value the integer value
   */
  void setInt(String key, int value);

  /**
   * Sets a string value for the given key.
   *
   * @param key the string key
   * @param value the string value
   */
  void setString(String key, String value);

  /**
   * Sets an object value for the given key.
   *
   * @param key the string key
   * @param value the object value
   */
  void setString(String key, Object value);

  /**
   * Removes a value by the given key.
   *
   * @param key a string key
   * @return true if the record was deleted successfully, false otherwise
   */
  boolean removeKey(String key);
}
