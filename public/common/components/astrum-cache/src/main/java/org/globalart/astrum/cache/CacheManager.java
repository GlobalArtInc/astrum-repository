
package org.globalart.astrum.cache;

import javax.cache.expiry.Duration;

/**
 * The cache manager to create the {@link AstrumCache}.
 * This is the lightweight version of the {@link javax.cache.Cache} which may use persistent cache
 * and if that is not required one should use {@link CacheHelper} then.
 *
 * @param <K> the type of key
 * @param <V> the type of value
 */
public interface CacheManager<K, V>
{
  /**
   * Creates the {@link AstrumCache}. It may be a distributed or a local cache depending on a Astrum mode.
   *
   * @param cacheName the name of the cache.
   * @param keyType the expected key type.
   * @param valueType the expected value type.
   * @param expiryAfter the expiration policy of the cache.
   * @return the newly created {@link AstrumCache}
   */
  AstrumCache<K, V> getCache(String cacheName, Class<K> keyType, Class<V> valueType, Duration expiryAfter);

  /**
   * Destroys the {@link AstrumCache} specified by cacheName.
   */
  void destroyCache(String cacheName);
}
