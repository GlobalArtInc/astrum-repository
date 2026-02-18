
package org.globalart.astrum.repository.cache.internal;

import org.globalart.astrum.repository.cache.NegativeCacheKey;

import static com.google.common.base.Preconditions.checkNotNull;

// TODO: implement Externalizable

/**
 * A path based {@link NegativeCacheKey}.
 *
 * @since 3.0
 */
public class PathNegativeCacheKey
    implements NegativeCacheKey
{
  private final String path;

  public PathNegativeCacheKey(final String path) {
    this.path = checkNotNull(path);
  }

  /**
   * @param key child key
   * @return true if child key path starts with this key path
   */
  @Override
  public boolean isParentOf(final NegativeCacheKey key) {
    checkNotNull(key);
    return path.endsWith("/")
        && key instanceof PathNegativeCacheKey
        && ((PathNegativeCacheKey) key).path.startsWith(path);
  }

  @Override
  public boolean equals(final Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }

    PathNegativeCacheKey that = (PathNegativeCacheKey) o;

    return path.equals(that.path);
  }

  @Override
  public int hashCode() {
    return path.hashCode();
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "path='" + path + '\'' +
        '}';
  }

}
