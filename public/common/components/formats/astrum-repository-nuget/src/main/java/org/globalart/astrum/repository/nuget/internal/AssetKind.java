package org.globalart.astrum.repository.nuget.internal;

import org.globalart.astrum.repository.cache.CacheControllerHolder;
import org.globalart.astrum.repository.cache.CacheControllerHolder.CacheType;

public enum AssetKind
{
  SERVICE_INDEX(CacheControllerHolder.METADATA),
  PACKAGE_VERSIONS(CacheControllerHolder.METADATA),
  REGISTRATION(CacheControllerHolder.METADATA),
  NUSPEC(CacheControllerHolder.CONTENT),
  PACKAGE_FILE(CacheControllerHolder.CONTENT),
  PACKAGE(CacheControllerHolder.CONTENT),
  V2_FEED(CacheControllerHolder.METADATA);

  private final CacheType cacheType;

  AssetKind(final CacheType cacheType) {
    this.cacheType = cacheType;
  }

  public CacheType getCacheType() {
    return cacheType;
  }
}
