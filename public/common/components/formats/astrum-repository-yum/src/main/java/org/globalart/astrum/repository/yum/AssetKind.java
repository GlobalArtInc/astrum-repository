package org.globalart.astrum.repository.yum;

import javax.annotation.Nonnull;

import org.globalart.astrum.repository.cache.CacheControllerHolder;
import org.globalart.astrum.repository.cache.CacheControllerHolder.CacheType;

public enum AssetKind
{
  REPOMD(CacheControllerHolder.METADATA),
  REPODATA(CacheControllerHolder.METADATA),
  RPM(CacheControllerHolder.CONTENT),
  OTHER(CacheControllerHolder.CONTENT);

  private final CacheType cacheType;

  AssetKind(final CacheType cacheType) {
    this.cacheType = cacheType;
  }

  @Nonnull
  public CacheType getCacheType() {
    return cacheType;
  }

  public boolean isMetadata() {
    return this == REPOMD || this == REPODATA;
  }
}
