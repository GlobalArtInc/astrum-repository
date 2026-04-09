
package org.globalart.astrum.repository.helm.internal;

import javax.annotation.Nonnull;

import org.globalart.astrum.repository.cache.CacheControllerHolder;
import org.globalart.astrum.repository.cache.CacheControllerHolder.CacheType;

/**
 * Asset kinds for Helm
 *
 * Examples of all can be found at: https://github.com/kubernetes/helm/blob/master/docs/chart_repository.md#the-chart-repository-structure
 * @since 3.28
 */
public enum AssetKind
{
  HELM_INDEX(CacheControllerHolder.METADATA, ".yaml"),
  HELM_PROVENANCE(CacheControllerHolder.CONTENT, ".tgz.prov"),
  HELM_PACKAGE(CacheControllerHolder.CONTENT, ".tgz");

  private final CacheType cacheType;
  private final String extension;

  AssetKind(final CacheType cacheType, final String extension) {
    this.cacheType = cacheType;
    this.extension = extension;
  }

  public static AssetKind getAssetKindByFileName(final String name) {
    if (name.endsWith(HELM_PACKAGE.getExtension())) {
      return AssetKind.HELM_PACKAGE;
    }
    else if (name.endsWith(HELM_PROVENANCE.getExtension())) {
      return AssetKind.HELM_PROVENANCE;
    }
    return AssetKind.HELM_INDEX;
  }

  @Nonnull
  public CacheType getCacheType() {
    return cacheType;
  }

  @Nonnull
  public String getExtension() {
    return extension;
  }
}
