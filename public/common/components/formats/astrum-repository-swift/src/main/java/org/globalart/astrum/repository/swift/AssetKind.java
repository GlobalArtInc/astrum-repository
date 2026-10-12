package org.globalart.astrum.repository.swift;

import javax.annotation.Nonnull;

import org.globalart.astrum.repository.cache.CacheControllerHolder;
import org.globalart.astrum.repository.cache.CacheControllerHolder.CacheType;

public enum AssetKind
{
  RELEASES(CacheControllerHolder.METADATA),
  RELEASE(CacheControllerHolder.METADATA),
  MANIFEST(CacheControllerHolder.CONTENT),
  ARCHIVE(CacheControllerHolder.CONTENT);

  private final CacheType cacheType;

  AssetKind(final CacheType cacheType) {
    this.cacheType = cacheType;
  }

  @Nonnull
  public CacheType getCacheType() {
    return cacheType;
  }
}
