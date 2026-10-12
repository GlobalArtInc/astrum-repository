package org.globalart.astrum.repository.terraform;

import javax.annotation.Nonnull;

import org.globalart.astrum.repository.cache.CacheControllerHolder;
import org.globalart.astrum.repository.cache.CacheControllerHolder.CacheType;

public enum AssetKind
{
  MODULE_VERSIONS(CacheControllerHolder.METADATA),
  MODULE_ARCHIVE(CacheControllerHolder.CONTENT),
  PROVIDER_VERSIONS(CacheControllerHolder.METADATA),
  PROVIDER_DOWNLOAD(CacheControllerHolder.METADATA),
  PROVIDER_ARCHIVE(CacheControllerHolder.CONTENT),
  PROVIDER_SHASUMS(CacheControllerHolder.CONTENT),
  PROVIDER_SIGNATURE(CacheControllerHolder.CONTENT),
  PROVIDER_KEY(CacheControllerHolder.CONTENT),
  PROVIDER_MANIFEST(CacheControllerHolder.CONTENT);

  private final CacheType cacheType;

  AssetKind(final CacheType cacheType) {
    this.cacheType = cacheType;
  }

  @Nonnull
  public CacheType getCacheType() {
    return cacheType;
  }
}
