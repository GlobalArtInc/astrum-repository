package org.globalart.astrum.repository.yum.internal.proxy;

import java.io.IOException;
import java.io.InputStream;
import java.util.Map;
import java.util.Optional;

import javax.annotation.Nonnull;
import javax.annotation.Nullable;

import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.cache.CacheController;
import org.globalart.astrum.repository.content.AssetBlob;
import org.globalart.astrum.repository.content.facet.ContentProxyFacetSupport;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.http.HttpMethods;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.repository.yum.AssetKind;
import org.globalart.astrum.repository.yum.YumContentFacet;
import org.globalart.astrum.repository.yum.YumFormat;
import org.globalart.astrum.repository.yum.internal.YumPaths;
import org.globalart.astrum.repository.yum.internal.repodata.Repomd;
import org.globalart.astrum.repository.yum.internal.repodata.RepomdData;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.yum.internal.YumPaths.requestPath;

@Facet.Exposed
@Component
@Qualifier(YumFormat.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class YumProxyFacet
    extends ContentProxyFacetSupport
{
  private static final Map<String, String> CHECKSUM_NAMES = Map.of(
      "sha", "sha1",
      "sha1", "sha1",
      "sha256", "sha256",
      "sha512", "sha512",
      "md5", "md5");

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
    String path = requestPath(context);
    Content stored = content().putProxied(path, assetKind, content);
    if (assetKind == AssetKind.REPOMD) {
      invalidateChangedMetadata(path, stored);
    }
    return stored;
  }

  @Override
  protected String getUrl(@Nonnull final Context context) {
    return requestPath(context).substring(1);
  }

  public Content repomd(final Context context, final String base) throws IOException {
    Request request = new Request.Builder()
        .action(HttpMethods.GET)
        .path(YumPaths.repomdPath(base))
        .build();
    Context repomdContext = new Context(getRepository(), request);
    repomdContext.getAttributes().set(AssetKind.class, AssetKind.REPOMD);
    return get(repomdContext);
  }

  void invalidateChangedMetadata(final String repomdPath, final Content repomdContent) {
    Optional<String> base = YumPaths.baseOfMetadata(repomdPath);
    if (base.isEmpty()) {
      return;
    }
    Repomd repomd;
    try (InputStream in = repomdContent.openInputStream()) {
      repomd = Repomd.parse(in);
    }
    catch (IOException e) {
      log.debug("Unable to read {} of repository {}: {}", repomdPath, getRepository().getName(), e.getMessage());
      return;
    }
    YumContentFacet content = content();
    for (RepomdData data : repomd.data()) {
      if (data.locationBase() != null) {
        continue;
      }
      String path;
      try {
        path = YumPaths.normalize(base.get() + "/" + data.location());
      }
      catch (IllegalArgumentException e) {
        continue;
      }
      Optional<FluentAsset> cached = content.getAsset(path);
      if (cached.isPresent() && !matches(cached.get(), data)) {
        log.debug("Marking {} of repository {} stale as {} lists another checksum", path, getRepository().getName(),
            repomdPath);
        cached.get().markAsStale();
      }
    }
  }

  private static boolean matches(final FluentAsset asset, final RepomdData data) {
    String algorithm = CHECKSUM_NAMES.get(data.checksumType().toLowerCase());
    if (algorithm == null) {
      return false;
    }
    return asset.blob()
        .map(AssetBlob::checksums)
        .map(checksums -> checksums.get(algorithm))
        .map(checksum -> checksum.equalsIgnoreCase(data.checksum()))
        .orElse(false);
  }

  private YumContentFacet content() {
    return getRepository().facet(YumContentFacet.class);
  }
}
