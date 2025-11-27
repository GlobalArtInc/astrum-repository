
package org.globalart.astrum.cache;

import java.util.function.BiConsumer;

import javax.cache.Cache;
import javax.cache.CacheManager;
import javax.cache.configuration.Factory;
import javax.cache.expiry.ExpiryPolicy;

/**
 * Abstracts cache configurations to make using different underlying implementations easier
 * 
 * @since 3.14
 */
public interface CacheBuilder<K, V>
{
  String getName();

  Class<K> getKeyType();

  Class<V> getValueType();

  CacheBuilder<K, V> name(String name);

  CacheBuilder<K, V> cacheSize(int size);

  CacheBuilder<K, V> expiryFactory(Factory<? extends ExpiryPolicy> expiryFactory);

  CacheBuilder<K, V> storeByValue(boolean storeByValue);

  CacheBuilder<K, V> managementEnabled(boolean enabled);

  CacheBuilder<K, V> statisticsEnabled(boolean enabled);

  CacheBuilder<K, V> keyType(Class<K> keyType);

  CacheBuilder<K, V> valueType(Class<V> valueType);

  CacheBuilder<K, V> persister(BiConsumer<K, V> persister);

  Cache<K, V> build(CacheManager manager);
}
