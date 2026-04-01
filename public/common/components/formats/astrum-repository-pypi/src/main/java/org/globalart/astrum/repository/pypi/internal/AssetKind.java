
package org.globalart.astrum.repository.pypi.internal;

import javax.annotation.Nonnull;

import org.globalart.astrum.repository.cache.CacheControllerHolder;
import org.globalart.astrum.repository.cache.CacheControllerHolder.CacheType;

/**
 * Asset kinds for PyPI.
 *
 * @since 3.1
 */
public enum AssetKind
{
  ROOT_INDEX(CacheControllerHolder.METADATA),
  INDEX(CacheControllerHolder.METADATA),
  PACKAGE(CacheControllerHolder.CONTENT),
  PACKAGE_SIGNATURE(CacheControllerHolder.METADATA);

  private final CacheType cacheType;

  AssetKind(final CacheType cacheType) {
    this.cacheType = cacheType;
  }

  @Nonnull
  public CacheType getCacheType() {
    return cacheType;
  }
}
