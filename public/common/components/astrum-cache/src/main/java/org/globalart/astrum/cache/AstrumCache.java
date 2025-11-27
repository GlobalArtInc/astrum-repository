
package org.globalart.astrum.cache;

import java.util.Optional;

/**
 * The lightweight version of the {@link javax.cache.Cache}.
 *
 * @param <K> the type of key
 * @param <V> the type of value
 */
public interface AstrumCache<K, V>
{
  /**
   * Gets an entry from the cache.
   *
   * @param key the key whose associated value is to be returned.
   * @return the element, or {@code Optional.empty()}, if it does not exist.
   */
  Optional<V> get(final K key);

  /**
   * Associates the specified value with the specified key in the cache. If the {@link AstrumCache} previously
   * contained a mapping for the key, the old value is replaced by the specified value.
   *
   * @param key key with which the specified value is to be associated
   * @param value value to be associated with the specified key
   */
  void put(final K key, final V value);

  /**
   * Removes the mapping for a key from this cache if it is present.
   *
   * @param key key whose mapping is to be removed from the cache
   */
  void remove(final K key);

  /**
   * Removes all the mappings from this cache.
   */
  void removeAll();
}
