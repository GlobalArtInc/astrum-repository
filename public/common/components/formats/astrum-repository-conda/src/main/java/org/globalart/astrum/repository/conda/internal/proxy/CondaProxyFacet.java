
package org.globalart.astrum.repository.conda.internal.proxy;

import java.io.IOException;

import javax.annotation.Nonnull;
import javax.annotation.Nullable;

import org.globalart.astrum.repository.cache.CacheController;
import org.globalart.astrum.repository.conda.AssetKind;
import org.globalart.astrum.repository.conda.CondaContentFacet;
import org.globalart.astrum.repository.conda.CondaFormat;
import org.globalart.astrum.repository.content.facet.ContentProxyFacetSupport;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.matchers.token.TokenMatcher;
import org.globalart.astrum.repository.view.payloads.TempBlob;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.conda.util.CondaPathUtils.assetPath;
import static org.globalart.astrum.repository.conda.util.CondaPathUtils.matcherState;
import static org.globalart.astrum.repository.conda.util.CondaPathUtils.name;
import static org.globalart.astrum.repository.conda.util.CondaPathUtils.subdir;
import static org.globalart.astrum.repository.conda.util.CondaPathUtils.version;

@Component
@Qualifier(CondaFormat.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class CondaProxyFacet
    extends ContentProxyFacetSupport
{
  @Nullable
  @Override
  protected Content getCachedContent(final Context context) {
    return content().getAsset(assetPath(context))
        .map(FluentAsset::download)
        .orElse(null);
  }

  @Override
  protected Content store(final Context context, final Content content) throws IOException {
    AssetKind assetKind = getAssetKind(context);
    String path = assetPath(context);
    try (TempBlob tempBlob = content().getTempBlob(content)) {
      FluentAsset asset;
      if (assetKind.isPackage()) {
        TokenMatcher.State state = matcherState(context);
        asset = content().putPackage(path, assetKind, subdir(state), name(state), version(state), tempBlob, null);
      }
      else {
        asset = content().putMetadata(path, assetKind, tempBlob);
      }
      return asset.markAsCached(content).download();
    }
  }

  @Override
  protected String getUrl(@Nonnull final Context context) {
    return context.getRequest().getPath().substring(1);
  }

  @Nonnull
  @Override
  protected CacheController getCacheController(@Nonnull final Context context) {
    return cacheControllerHolder.require(getAssetKind(context).getCacheType());
  }

  private AssetKind getAssetKind(final Context context) {
    return context.getAttributes().require(AssetKind.class);
  }

  private CondaContentFacet content() {
    return facet(CondaContentFacet.class);
  }
}
