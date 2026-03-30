
package org.globalart.astrum.repository.npm.internal;

import javax.annotation.Nonnull;

import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.cache.CacheControllerHolder;
import org.globalart.astrum.repository.cache.CacheControllerHolder.CacheType;

import static com.google.common.base.Preconditions.checkNotNull;

@Facet.Exposed
public interface NpmProxyFacet
    extends Facet
{

  void invalidateProxyCaches();

  enum ProxyTarget
  {
    SEARCH_V1_RESULTS(CacheControllerHolder.METADATA),
    PACKAGE(CacheControllerHolder.METADATA),
    DIST_TAGS(CacheControllerHolder.METADATA),
    TARBALL(CacheControllerHolder.CONTENT);

    private final CacheType cacheType;

    ProxyTarget(final CacheType cacheType) {
      this.cacheType = checkNotNull(cacheType);
    }

    @Nonnull
    public CacheType getCacheType() {
      return cacheType;
    }
  }
}
