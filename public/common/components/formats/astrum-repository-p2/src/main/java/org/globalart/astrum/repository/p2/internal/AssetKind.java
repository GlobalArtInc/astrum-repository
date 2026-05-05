
package org.globalart.astrum.repository.p2.internal;

import javax.annotation.Nonnull;

import org.globalart.astrum.repository.cache.CacheControllerHolder;
import org.globalart.astrum.repository.cache.CacheControllerHolder.CacheType;

public enum AssetKind
{
  P2_INDEX(CacheControllerHolder.METADATA),
  ARTIFACTS_METADATA(CacheControllerHolder.METADATA),
  CONTENT_METADATA(CacheControllerHolder.METADATA),
  COMPOSITE_ARTIFACTS(CacheControllerHolder.METADATA),
  COMPOSITE_CONTENT(CacheControllerHolder.METADATA),
  BUNDLE(CacheControllerHolder.CONTENT),
  BINARY_BUNDLE(CacheControllerHolder.CONTENT);

  private final CacheType cacheType;

  AssetKind(final CacheType cacheType) {
    this.cacheType = cacheType;
  }

  @Nonnull
  public CacheType getCacheType() {
    return cacheType;
  }
}
