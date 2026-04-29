
package org.globalart.astrum.repository.conan;

import javax.annotation.Nonnull;

import org.globalart.astrum.repository.cache.CacheControllerHolder;
import org.globalart.astrum.repository.cache.CacheControllerHolder.CacheType;

public enum AssetKind
{
  RECIPE_FILE(CacheControllerHolder.CONTENT),
  RECIPE_MANIFEST(CacheControllerHolder.CONTENT),
  PACKAGE_FILE(CacheControllerHolder.CONTENT),
  PACKAGE_MANIFEST(CacheControllerHolder.CONTENT),
  RECIPE_LATEST(CacheControllerHolder.METADATA),
  RECIPE_REVISIONS(CacheControllerHolder.METADATA),
  RECIPE_FILES(CacheControllerHolder.METADATA),
  PACKAGES_SEARCH(CacheControllerHolder.METADATA),
  PACKAGE_LATEST(CacheControllerHolder.METADATA),
  PACKAGE_REVISIONS(CacheControllerHolder.METADATA),
  PACKAGE_FILES(CacheControllerHolder.METADATA),
  DOWNLOAD_URLS(CacheControllerHolder.METADATA),
  DIGEST(CacheControllerHolder.METADATA),
  SNAPSHOT(CacheControllerHolder.METADATA);

  private final CacheType cacheType;

  AssetKind(final CacheType cacheType) {
    this.cacheType = cacheType;
  }

  @Nonnull
  public CacheType getCacheType() {
    return cacheType;
  }

  public boolean isFile() {
    return this == RECIPE_FILE || this == RECIPE_MANIFEST || this == PACKAGE_FILE || this == PACKAGE_MANIFEST;
  }
}
