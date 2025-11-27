
package org.globalart.astrum.cache;

import javax.annotation.Nullable;
import javax.cache.Cache;
import javax.cache.CacheManager;
import javax.cache.configuration.Factory;
import javax.cache.configuration.MutableConfiguration;
import javax.cache.expiry.ExpiryPolicy;
import org.springframework.beans.factory.annotation.Autowired;
import jakarta.inject.Provider;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * @since 3.6
 */
@SuppressWarnings("rawtypes")
@Component
public class CacheHelper
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final Provider<CacheManager> cacheManagerProvider;

  private final Provider<CacheBuilder> cacheBuilderProvider;

  @Autowired
  public CacheHelper(
      final Provider<CacheManager> cacheManagerProvider,
      final Provider<CacheBuilder> cacheBuilderProvider)
  {
    this.cacheManagerProvider = checkNotNull(cacheManagerProvider);
    this.cacheBuilderProvider = checkNotNull(cacheBuilderProvider);
  }

  private CacheManager manager() {
    return cacheManagerProvider.get();
  }

  @SuppressWarnings("unchecked")
  public <K, V> CacheBuilder<K, V> builder() {
    return cacheBuilderProvider.get();
  }

  public synchronized <K, V> Cache<K, V> maybeCreateCache(
      final String name,
      final MutableConfiguration<K, V> mutableConfiguration)
  {
    checkNotNull(name);
    checkNotNull(mutableConfiguration);

    Cache<K, V> cache = manager()
        .getCache(name, mutableConfiguration.getKeyType(), mutableConfiguration.getValueType());

    if (cache == null) {
      cache = manager().createCache(name, mutableConfiguration);
      log.debug("Created cache: {}", cache);
    }
    else {
      log.debug("Re-using existing cache: {}", cache);
    }

    return cache;
  }

  public synchronized <K, V> Cache<K, V> getOrCreate(final CacheBuilder<K, V> builder) {
    checkNotNull(builder);

    Cache<K, V> cache = manager().getCache(builder.getName(), builder.getKeyType(), builder.getValueType());

    if (cache == null) {
      cache = builder.build(manager());
      log.debug("Created cache: {}", cache);
    }
    else {
      log.debug("Re-using existing cache: {}", cache);
    }

    return cache;
  }

  public synchronized <K, V> Cache<K, V> maybeCreateCache(
      final String name,
      final Factory<? extends ExpiryPolicy> expiryPolicyFactory)
  {
    return maybeCreateCache(name, null, null, expiryPolicyFactory);
  }

  public synchronized <K, V> Cache<K, V> maybeCreateCache(
      final String name,
      @Nullable final Class<K> keyType,
      @Nullable final Class<V> valueType,
      final Factory<? extends ExpiryPolicy> expiryPolicyFactory)
  {
    return maybeCreateCache(name, createCacheConfig(keyType, valueType, expiryPolicyFactory));
  }

  public static <K, V> MutableConfiguration<K, V> createCacheConfig(
      @Nullable final Class<K> keyType,
      @Nullable final Class<V> valueType,
      final Factory<? extends ExpiryPolicy> expiryPolicyFactory)
  {
    MutableConfiguration<K, V> config = new MutableConfiguration<K, V>()
        .setStoreByValue(false)
        .setExpiryPolicyFactory(expiryPolicyFactory)
        .setManagementEnabled(true)
        .setStatisticsEnabled(true);

    if (keyType != null && valueType != null) {
      config.setTypes(keyType, valueType);
    }
    return config;
  }

  public synchronized void maybeDestroyCache(final String name) {
    manager().destroyCache(name);
    log.debug("Destroyed cache: {}", name);
  }
}
