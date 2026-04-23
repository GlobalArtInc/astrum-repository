
package org.globalart.astrum.repository.alpine.internal.proxy;

import java.io.IOException;

import javax.annotation.Nonnull;
import javax.annotation.Nullable;

import org.globalart.astrum.repository.alpine.AlpineContentFacet;
import org.globalart.astrum.repository.alpine.AlpineFormat;
import org.globalart.astrum.repository.alpine.AssetKind;
import org.globalart.astrum.repository.cache.CacheController;
import org.globalart.astrum.repository.content.facet.ContentProxyFacetSupport;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Context;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.alpine.internal.AlpinePathUtils.requestPath;

@Component
@Qualifier(AlpineFormat.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class AlpineProxyFacet
    extends ContentProxyFacetSupport
{
  @Nullable
  @Override
  protected Content getCachedContent(final Context context) throws IOException {
    return content().get(requestPath(context)).orElse(null);
  }

  @Nonnull
  @Override
  protected CacheController getCacheController(@Nonnull final Context context) {
    AssetKind assetKind = context.getAttributes().require(AssetKind.class);
    return cacheControllerHolder.require(assetKind.getCacheType());
  }

  @Override
  protected Content store(final Context context, final Content content) throws IOException {
    AssetKind assetKind = context.getAttributes().require(AssetKind.class);
    return content().putProxied(requestPath(context), assetKind, content);
  }

  @Override
  protected String getUrl(@Nonnull final Context context) {
    return requestPath(context).substring(1);
  }

  private AlpineContentFacet content() {
    return getRepository().facet(AlpineContentFacet.class);
  }
}
