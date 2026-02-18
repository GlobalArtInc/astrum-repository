
package org.globalart.astrum.repository.cache.internal;

import org.globalart.astrum.repository.cache.NegativeCacheKey;

import static com.google.common.base.Preconditions.checkNotNull;

// TODO: implement Externalizable

/**
 * A simple value based {@link NegativeCacheKey}.
 *
 * @since 3.0
 */
public class ValueNegativeCacheKey
    implements NegativeCacheKey
{
  private final String value;

  public ValueNegativeCacheKey(final String value) {
    this.value = checkNotNull(value);
  }

  /**
   * @param key child key
   * @return false
   */
  @Override
  public boolean isParentOf(final NegativeCacheKey key) {
    checkNotNull(key);
    return false;
  }

  @Override
  public boolean equals(final Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }

    ValueNegativeCacheKey that = (ValueNegativeCacheKey) o;

    return value.equals(that.value);
  }

  @Override
  public int hashCode() {
    return value.hashCode();
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "value='" + value + '\'' +
        '}';
  }

}
