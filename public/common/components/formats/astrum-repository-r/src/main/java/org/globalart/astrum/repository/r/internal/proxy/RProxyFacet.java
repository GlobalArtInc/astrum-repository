
package org.globalart.astrum.repository.r.internal.proxy;

import java.io.IOException;

import javax.annotation.Nonnull;
import javax.annotation.Nullable;

import org.globalart.astrum.repository.cache.CacheController;
import org.globalart.astrum.repository.content.facet.ContentProxyFacetSupport;
import org.globalart.astrum.repository.r.AssetKind;
import org.globalart.astrum.repository.r.RContentFacet;
import org.globalart.astrum.repository.r.RFormat;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Context;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.r.internal.util.RPathUtils.requestPath;

@Component
@Qualifier(RFormat.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class RProxyFacet
    extends ContentProxyFacetSupport
{
  @Nullable
  @Override
  protected Content getCachedContent(final Context context) {
    return content().get(requestPath(context)).orElse(null);
  }

  @Nonnull
  @Override
  protected CacheController getCacheController(@Nonnull final Context context) {
    return cacheControllerHolder.require(getAssetKind(context).getCacheType());
  }

  @Override
  protected Content store(final Context context, final Content content) throws IOException {
    return content().putProxied(requestPath(context), getAssetKind(context), content);
  }

  @Override
  protected String getUrl(@Nonnull final Context context) {
    return requestPath(context).substring(1);
  }

  private static AssetKind getAssetKind(final Context context) {
    return context.getAttributes().require(AssetKind.class);
  }

  private RContentFacet content() {
    return facet(RContentFacet.class);
  }
}
