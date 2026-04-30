
package org.globalart.astrum.repository.cocoapods.internal;

import org.globalart.astrum.repository.cache.CacheControllerHolder.CacheType;

import static org.globalart.astrum.repository.cache.CacheControllerHolder.CONTENT;
import static org.globalart.astrum.repository.cache.CacheControllerHolder.METADATA;

public enum AssetKind
{
  SPEC(METADATA),
  CDN_METADATA(METADATA),
  POD(CONTENT);

  private final CacheType cacheType;

  AssetKind(final CacheType cacheType) {
    this.cacheType = cacheType;
  }

  public CacheType getCacheType() {
    return cacheType;
  }
}
