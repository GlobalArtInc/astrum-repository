
package org.globalart.astrum.cache.internal;

import java.util.Optional;
import javax.cache.Cache;

import org.globalart.astrum.cache.AstrumCache;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * The implementation of the {@link AstrumCache}.
 *
 * @param <K> the type of key
 * @param <V> the type of value
 */
public class LocalCache<K, V>
    implements AstrumCache<K, V>
{
  private final Cache<K, V> cache;

  public LocalCache(final Cache<K, V> cache) {
    this.cache = checkNotNull(cache);
  }

  @Override
  public Optional<V> get(final K key) {
    return Optional.ofNullable(cache.get(key));
  }

  @Override
  public void put(final K key, final V value) {
    cache.put(key, value);
  }

  @Override
  public void remove(final K key) {
    cache.remove(key);
  }

  @Override
  public void removeAll() {
    cache.removeAll();
  }
}
