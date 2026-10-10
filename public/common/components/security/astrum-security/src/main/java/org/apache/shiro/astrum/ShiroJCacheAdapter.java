
package org.apache.shiro.astrum;

import java.util.ArrayList;
import java.util.Collection;
import java.util.Collections;
import java.util.HashSet;
import java.util.Set;

import javax.cache.Cache.Entry;

import com.google.common.collect.Iterables;
import org.apache.shiro.cache.Cache;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Shiro {@link javax.cache.Cache} to {@link Cache} adapter.
 *
 * @since 3.0
 */
public class ShiroJCacheAdapter<K, V>
    implements Cache<K, V>
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final javax.cache.Cache<K, V> cache;

  private final String name;

  public ShiroJCacheAdapter(final javax.cache.Cache<K, V> cache) {
    this.cache = checkNotNull(cache);
    this.name = cache.getName();
  }

  @Override
  public V get(final K key) {
    return cache.get(key);
  }

  @Override
  public V put(final K key, final V value) {
    return cache.getAndPut(key, value);
  }

  @Override
  public V remove(final K key) {
    return cache.getAndRemove(key);
  }

  // NOTE: This appears unused in Shiro, but used by Astrum
  @Override
  public void clear() {
    cache.clear();
  }

  // NOTE: This appears unused in Shiro.
  @Override
  public int size() {
    return Iterables.size(cache);
  }

  // NOTE: This appears unused in Shiro.
  @Override
  public Set<K> keys() {
    Set<K> keys = new HashSet<>();
    for (Entry<K, V> entry : cache) {
      keys.add(entry.getKey());
    }
    return Collections.unmodifiableSet(keys);
  }

  @Override
  public Collection<V> values() {
    Collection<V> values = new ArrayList<>();
    for (Entry<K, V> entry : cache) {
      values.add(entry.getValue());
    }
    return Collections.unmodifiableCollection(values);
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "cache=" + cache +
        ", name='" + name + '\'' +
        '}';
  }
}
