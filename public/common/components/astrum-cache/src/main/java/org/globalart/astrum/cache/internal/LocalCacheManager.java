
package org.globalart.astrum.cache.internal;

import javax.cache.expiry.CreatedExpiryPolicy;
import javax.cache.expiry.Duration;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.cache.CacheHelper;
import org.globalart.astrum.cache.CacheManager;
import org.globalart.astrum.cache.AstrumCache;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * The cache manager which creates the {@link LocalCache}.
 * The lightweight version of the {@link javax.cache.Cache} which should be used in the single mode only.
 *
 * @param <K> the type of key
 * @param <V> the type of value
 */
@Component
public class LocalCacheManager<K, V>
    implements CacheManager<K, V>
{
  private final CacheHelper cacheHelper;

  @Autowired
  public LocalCacheManager(final CacheHelper cacheHelper) {
    this.cacheHelper = checkNotNull(cacheHelper);
  }

  /**
   * Warning:
   * If the cache specified by cacheName already exists, then the existing cache is returned and
   * the expiryAfter is ignored.
   * The cache must be destroyed before a new expiryAfter can be specified.
   */
  @Override
  public AstrumCache<K, V> getCache(
      final String cacheName,
      final Class<K> keyType,
      final Class<V> valueType,
      final Duration expiryAfter)
  {
    return new LocalCache<>(
        cacheHelper.maybeCreateCache(cacheName, keyType, valueType, CreatedExpiryPolicy.factoryOf(expiryAfter)));
  }

  @Override
  public void destroyCache(final String cacheName) {
    cacheHelper.maybeDestroyCache(cacheName);
  }
}
