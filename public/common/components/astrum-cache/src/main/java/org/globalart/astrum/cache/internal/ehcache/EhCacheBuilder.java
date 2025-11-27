
package org.globalart.astrum.cache.internal.ehcache;

import java.time.Duration;
import java.time.temporal.ChronoUnit;
import java.util.function.Supplier;

import javax.cache.Cache;
import javax.cache.CacheManager;

import org.globalart.astrum.cache.AbstractCacheBuilder;
import org.globalart.astrum.cache.CacheBuilder;

import org.ehcache.config.CacheRuntimeConfiguration;
import org.ehcache.config.builders.CacheConfigurationBuilder;
import org.ehcache.config.builders.ResourcePoolsBuilder;
import org.ehcache.event.CacheEventListener;
import org.ehcache.event.EventFiring;
import org.ehcache.event.EventOrdering;
import org.ehcache.event.EventType;
import org.ehcache.expiry.ExpiryPolicy;
import org.ehcache.jsr107.Eh107Configuration;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * EhCache JCache {@link CacheBuilder}.
 *
 * @since 3.14
 */
@Component
@Qualifier("ehcache")
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class EhCacheBuilder<K, V>
    extends AbstractCacheBuilder<K, V>
{
  @Override
  @SuppressWarnings("unchecked")
  public Cache<K, V> build(final CacheManager manager) {
    checkNotNull(manager);
    checkNotNull(keyType);
    checkNotNull(valueType);
    checkNotNull(name);
    checkNotNull(expiryFactory);

    CacheConfigurationBuilder<K, V> builder = CacheConfigurationBuilder.newCacheConfigurationBuilder(
        keyType,
        valueType,
        ResourcePoolsBuilder.heap(cacheSize));

    builder.withExpiry(mapToEhCacheExpiry(expiryFactory.create()));

    Cache<K, V> cache = manager.createCache(name, Eh107Configuration.fromEhcacheCacheConfiguration(builder));

    manager.enableStatistics(name, statisticsEnabled);
    manager.enableManagement(name, managementEnabled);

    if (persister != null) {
      CacheEventListener<K, V> listener = cacheEvent -> persister.accept(cacheEvent.getKey(), cacheEvent.getOldValue());

      Eh107Configuration<K, V> configuration = cache.getConfiguration(Eh107Configuration.class);
      configuration.unwrap(CacheRuntimeConfiguration.class)
          .registerCacheEventListener(listener, EventOrdering.UNORDERED, EventFiring.ASYNCHRONOUS,
              EventType.EVICTED, EventType.REMOVED, EventType.EXPIRED);
    }

    return cache;
  }

  private ExpiryPolicy<K, V> mapToEhCacheExpiry(final javax.cache.expiry.ExpiryPolicy policy) {
    return new ExpiryPolicy<K, V>()
    {
      @Override
      public Duration getExpiryForCreation(final K key, final V value) {
        return toJavaDuration(policy.getExpiryForCreation());
      }

      @Override
      public Duration getExpiryForAccess(final K key, final Supplier<? extends V> value) {
        return toJavaDuration(policy.getExpiryForAccess());
      }

      @Override
      public Duration getExpiryForUpdate(final K key, final Supplier<? extends V> oldValue, final V newValue) {
        return toJavaDuration(policy.getExpiryForUpdate());
      }

      private Duration toJavaDuration(final javax.cache.expiry.Duration duration) {
        if (duration.isEternal()) {
          return Duration.of(1, ChronoUnit.FOREVER);
        }
        ChronoUnit chronoUnit = null;
        switch (duration.getTimeUnit()) {
          case DAYS:
            chronoUnit = ChronoUnit.DAYS;
            break;
          case HOURS:
            chronoUnit = ChronoUnit.HOURS;
            break;
          case MICROSECONDS:
            chronoUnit = ChronoUnit.MICROS;
            break;
          case MILLISECONDS:
            chronoUnit = ChronoUnit.MILLIS;
            break;
          case MINUTES:
            chronoUnit = ChronoUnit.MINUTES;
            break;
          case NANOSECONDS:
            chronoUnit = ChronoUnit.NANOS;
            break;
          case SECONDS:
            chronoUnit = ChronoUnit.SECONDS;
            break;
        }
        return Duration.of(duration.getDurationAmount(), chronoUnit);
      }
    };
  }
}
