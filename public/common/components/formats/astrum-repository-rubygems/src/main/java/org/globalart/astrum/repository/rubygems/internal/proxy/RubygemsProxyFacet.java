package org.globalart.astrum.repository.rubygems.internal.proxy;

import java.io.IOException;
import java.io.InputStream;
import java.util.Map;

import javax.annotation.Nonnull;
import javax.annotation.Nullable;

import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.cache.CacheController;
import org.globalart.astrum.repository.content.facet.ContentProxyFacetSupport;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.rubygems.AssetKind;
import org.globalart.astrum.repository.rubygems.RubygemsFormat;
import org.globalart.astrum.repository.rubygems.internal.RubygemsContentFacet;
import org.globalart.astrum.repository.rubygems.internal.gem.GemPackage;
import org.globalart.astrum.repository.rubygems.internal.gem.GemSpec;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.payloads.TempBlob;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

@Facet.Exposed
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
@Qualifier(RubygemsFormat.NAME)
public class RubygemsProxyFacet
    extends ContentProxyFacetSupport
{
  @Nullable
  @Override
  protected Content getCachedContent(final Context context) {
    return content().getAsset(context.getRequest().getPath()).map(FluentAsset::download).orElse(null);
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
    String path = context.getRequest().getPath();
    RubygemsContentFacet contentFacet = content();
    try (TempBlob blob = contentFacet.ingest(content)) {
      FluentAsset asset = assetKind == AssetKind.GEM
          ? storeGem(path, blob)
          : contentFacet.putFile(path, assetKind, blob, Map.of());
      return asset.markAsCached(content).download();
    }
  }

  private FluentAsset storeGem(final String path, final TempBlob blob) {
    GemSpec spec = null;
    try (InputStream in = blob.get()) {
      spec = GemPackage.readSpec(in);
    }
    catch (IOException | IllegalArgumentException e) {
      log.warn("Unable to read the specification of {} cached by {}: {}", path, getRepository().getName(),
          e.getMessage());
    }
    if (spec == null) {
      return content().putFile(path, AssetKind.GEM, blob, Map.of());
    }
    if (!path.equals(spec.coordinates().gemPath())) {
      log.warn("Gem {} cached by {} contains {}", path, getRepository().getName(), spec.coordinates().fileName());
    }
    return content().putGem(path, spec, blob);
  }

  @Override
  protected String getUrl(@Nonnull final Context context) {
    return context.getRequest().getPath().substring(1);
  }

  private RubygemsContentFacet content() {
    return facet(RubygemsContentFacet.class);
  }
}
