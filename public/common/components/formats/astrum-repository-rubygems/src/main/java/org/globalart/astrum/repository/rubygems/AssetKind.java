package org.globalart.astrum.repository.rubygems;

import javax.annotation.Nonnull;

import org.globalart.astrum.repository.cache.CacheControllerHolder;
import org.globalart.astrum.repository.cache.CacheControllerHolder.CacheType;

public enum AssetKind
{
  GEM(CacheControllerHolder.CONTENT),
  GEMSPEC(CacheControllerHolder.CONTENT),
  INDEX(CacheControllerHolder.METADATA);

  private final CacheType cacheType;

  AssetKind(final CacheType cacheType) {
    this.cacheType = cacheType;
  }

  @Nonnull
  public CacheType getCacheType() {
    return cacheType;
  }
}
