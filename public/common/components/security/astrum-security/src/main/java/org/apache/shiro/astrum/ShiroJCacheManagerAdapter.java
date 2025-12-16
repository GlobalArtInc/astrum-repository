
package org.apache.shiro.astrum;

import java.util.Objects;
import java.util.Optional;

import javax.cache.expiry.CreatedExpiryPolicy;
import javax.cache.expiry.Duration;
import javax.cache.expiry.EternalExpiryPolicy;
import jakarta.inject.Provider;

import org.globalart.astrum.common.time.Time;
import org.globalart.astrum.cache.CacheHelper;

import com.google.common.annotations.VisibleForTesting;
import org.apache.shiro.cache.Cache;
import org.apache.shiro.cache.CacheManager;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.apache.shiro.session.mgt.eis.CachingSessionDAO.ACTIVE_SESSION_CACHE_NAME;

/**
 * Shiro {@link javax.cache.CacheManager} to {@link CacheManager} adapter.
 *
 * @since 3.0
 */
public class ShiroJCacheManagerAdapter
    implements CacheManager
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final Provider<CacheHelper> cacheHelperProvider;

  private final Provider<Time> defaultTimeToLive;

  public ShiroJCacheManagerAdapter(
      final Provider<CacheHelper> cacheHelperProvider,
      final Provider<Time> defaultTimeToLive)
  {
    this.cacheHelperProvider = checkNotNull(cacheHelperProvider);
    this.defaultTimeToLive = checkNotNull(defaultTimeToLive);
  }

  @Override
  public <K, V> Cache<K, V> getCache(final String name) {
    log.debug("Getting cache: {}", name);
    return new ShiroJCacheAdapter<>(this.<K, V>maybeCreateCache(name));
  }

  @VisibleForTesting
  <K, V> javax.cache.Cache<K, V> maybeCreateCache(final String name) {
    if (Objects.equals(ACTIVE_SESSION_CACHE_NAME, name)) {
      // shiro's session cache needs to never expire:
      // http://shiro.apache.org/session-management.html#ehcache-session-cache-configuration
      return cacheHelperProvider.get().maybeCreateCache(name, EternalExpiryPolicy.factoryOf());
    }
    else {
      Time timeToLive = Optional.ofNullable(System.getProperty(name + ".timeToLive"))
          .map(Time::parse)
          .orElse(defaultTimeToLive.get());
      return cacheHelperProvider.get()
          .maybeCreateCache(name,
              CreatedExpiryPolicy.factoryOf(new Duration(timeToLive.getUnit(), timeToLive.getValue())));
    }
  }
}
