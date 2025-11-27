
package org.globalart.astrum.cache;

import java.util.function.BiConsumer;

import javax.cache.configuration.Factory;
import javax.cache.expiry.ExpiryPolicy;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Abstracts cache builder to contain all of the getters/setters
 *
 * @since 3.14
 */
public abstract class AbstractCacheBuilder<K, V>
    implements CacheBuilder<K, V>
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  protected String name;

  protected Factory<? extends ExpiryPolicy> expiryFactory;

  protected int cacheSize = 10000;

  protected boolean storeByValue = false;

  protected boolean managementEnabled = true;

  protected boolean statisticsEnabled = true;

  protected Class<K> keyType;

  protected Class<V> valueType;

  protected BiConsumer<K, V> persister;

  @Override
  public String getName() {
    return this.name;
  }

  @Override
  public Class<K> getKeyType() {
    return this.keyType;
  }

  @Override
  public Class<V> getValueType() {
    return this.valueType;
  }

  @Override
  public CacheBuilder<K, V> name(final String name) {
    this.name = name;
    return this;
  }

  @Override
  public CacheBuilder<K, V> cacheSize(final int cacheSize) {
    this.cacheSize = cacheSize;
    return this;
  }

  @Override
  public CacheBuilder<K, V> expiryFactory(final Factory<? extends ExpiryPolicy> expiryFactory) {
    this.expiryFactory = expiryFactory;
    return this;
  }

  @Override
  public CacheBuilder<K, V> storeByValue(final boolean storeByValue) {
    this.storeByValue = storeByValue;
    return this;
  }

  @Override
  public CacheBuilder<K, V> managementEnabled(final boolean enabled) {
    this.managementEnabled = enabled;
    return this;
  }

  @Override
  public CacheBuilder<K, V> statisticsEnabled(final boolean enabled) {
    this.statisticsEnabled = enabled;
    return this;
  }

  @Override
  public CacheBuilder<K, V> keyType(final Class<K> keyType) {
    this.keyType = keyType;
    return this;
  }

  @Override
  public CacheBuilder<K, V> valueType(final Class<V> valueType) {
    this.valueType = valueType;
    return this;
  }

  @Override
  public CacheBuilder<K, V> persister(final BiConsumer<K, V> persister) {
    this.persister = persister;
    return this;
  }
}
