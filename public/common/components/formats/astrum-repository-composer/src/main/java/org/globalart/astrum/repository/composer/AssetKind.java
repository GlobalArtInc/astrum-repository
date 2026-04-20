
package org.globalart.astrum.repository.composer;

import org.globalart.astrum.repository.cache.CacheControllerHolder.CacheType;

import static org.globalart.astrum.repository.cache.CacheControllerHolder.CONTENT;
import static org.globalart.astrum.repository.cache.CacheControllerHolder.METADATA;

/**
 * Enumeration defining the valid asset kinds for a Composer-format repository.
 */
public enum AssetKind
{
  ZIPBALL(CONTENT),
  PACKAGES(METADATA),
  LIST(METADATA),
  PROVIDER(METADATA),
  PACKAGE(METADATA);

  private final CacheType cacheType;

  AssetKind(final CacheType cacheType) {
    this.cacheType = cacheType;
  }

  public CacheType getCacheType() {
    return cacheType;
  }
}
