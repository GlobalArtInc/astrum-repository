
package org.globalart.astrum.repository.golang.internal.content.proxy;

import javax.annotation.Nonnull;
import javax.annotation.Nullable;

import org.globalart.astrum.repository.cache.CacheController;
import org.globalart.astrum.repository.content.facet.ContentProxyFacetSupport;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.golang.AssetKind;
import org.globalart.astrum.repository.golang.internal.content.GoContentFacet;
import org.globalart.astrum.repository.golang.internal.util.GolangPathUtils;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.matchers.token.TokenMatcher;

import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class GoProxyFacet
    extends ContentProxyFacetSupport
{
  @Nullable
  @Override
  protected Content getCachedContent(final Context context) {
    return content().getAsset(getAssetPath(context)).map(FluentAsset::download).orElse(null);
  }

  @Nonnull
  @Override
  protected CacheController getCacheController(@Nonnull final Context context) {
    AssetKind assetKind = context.getAttributes().require(AssetKind.class);
    return cacheControllerHolder.require(assetKind.getCacheType());
  }

  @Override
  protected Content store(final Context context, final Content content) {
    AssetKind assetKind = context.getAttributes().require(AssetKind.class);
    String assetPath = getAssetPath(context);
    FluentAsset asset = switch (assetKind) {
      case INFO, MODULE, PACKAGE -> content().saveComponentAndAsset(assetPath, content, assetKind,
          GolangPathUtils.getAttributesFromMatcherState(GolangPathUtils.matcherState(context)));
      case LIST, LATEST -> content().saveAsset(assetPath, content, assetKind);
    };
    return asset.markAsCached(content).download();
  }

  @Override
  protected String getUrl(@Nonnull final Context context) {
    return context.getRequest().getPath().substring(1);
  }

  private String getAssetPath(final Context context) {
    AssetKind assetKind = context.getAttributes().require(AssetKind.class);
    TokenMatcher.State matcherState = GolangPathUtils.matcherState(context);
    return switch (assetKind) {
      case INFO, MODULE, PACKAGE -> GolangPathUtils.assetPath(matcherState);
      case LIST -> GolangPathUtils.listPath(matcherState);
      case LATEST -> GolangPathUtils.latestPath(matcherState);
    };
  }

  private GoContentFacet content() {
    return facet(GoContentFacet.class);
  }
}
