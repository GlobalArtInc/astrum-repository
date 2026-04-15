
package org.globalart.astrum.repository.golang;

import javax.annotation.Nonnull;

import org.globalart.astrum.repository.cache.CacheControllerHolder;
import org.globalart.astrum.repository.cache.CacheControllerHolder.CacheType;

public enum AssetKind
{
  PACKAGE(CacheControllerHolder.CONTENT),
  INFO(CacheControllerHolder.METADATA),
  LIST(CacheControllerHolder.METADATA),
  LATEST(CacheControllerHolder.METADATA),
  MODULE(CacheControllerHolder.METADATA);

  private final CacheType cacheType;

  AssetKind(final CacheType cacheType) {
    this.cacheType = cacheType;
  }

  @Nonnull
  public CacheType getCacheType() {
    return cacheType;
  }
}
