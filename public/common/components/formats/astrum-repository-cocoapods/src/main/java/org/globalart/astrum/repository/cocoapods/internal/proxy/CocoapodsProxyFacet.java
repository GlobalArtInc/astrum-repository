
package org.globalart.astrum.repository.cocoapods.internal.proxy;

import java.io.IOException;
import java.io.InputStream;
import java.net.URI;
import java.nio.charset.StandardCharsets;

import javax.annotation.Nonnull;
import javax.annotation.Nullable;

import org.globalart.astrum.repository.cache.CacheController;
import org.globalart.astrum.repository.cocoapods.CocoapodsContentFacet;
import org.globalart.astrum.repository.cocoapods.internal.AssetKind;
import org.globalart.astrum.repository.cocoapods.internal.CocoapodsFormat;
import org.globalart.astrum.repository.cocoapods.internal.PathUtils;
import org.globalart.astrum.repository.cocoapods.internal.proxy.SpecFileProcessor.SpecInfo;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.facet.ContentProxyFacetSupport;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.repository.view.matchers.token.TokenMatcher;
import org.globalart.astrum.repository.view.payloads.BytesPayload;
import org.globalart.astrum.repository.view.payloads.TempBlob;

import com.google.common.base.Strings;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.cocoapods.internal.AssetKind.SPEC;
import static org.globalart.astrum.repository.cocoapods.internal.CocoapodsFormat.FILENAME_KEY;
import static org.globalart.astrum.repository.cocoapods.internal.CocoapodsFormat.PACKAGE_NAME_KEY;
import static org.globalart.astrum.repository.cocoapods.internal.CocoapodsFormat.PACKAGE_VERSION_KEY;
import static org.globalart.astrum.repository.cocoapods.internal.CocoapodsFormat.removeInitialSlashFromPath;
import static org.globalart.astrum.repository.http.HttpMethods.GET;
import static org.globalart.astrum.repository.view.ContentTypes.APPLICATION_JSON;

@Component
@Qualifier(CocoapodsFormat.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class CocoapodsProxyFacet
    extends ContentProxyFacetSupport
{
  private final SpecFileProcessor specFileProcessor;

  @Autowired
  public CocoapodsProxyFacet(final SpecFileProcessor specFileProcessor) {
    this.specFileProcessor = checkNotNull(specFileProcessor);
  }

  @Override
  public Content get(final Context context) throws IOException {
    Content content = super.get(context);
    if (content != null && getAssetKind(context) == SPEC) {
      return transformSpecFile(content);
    }
    return content;
  }

  @Nullable
  @Override
  protected Content getCachedContent(final Context context) {
    return content().getAsset(getPath(context))
        .map(FluentAsset::download)
        .orElse(null);
  }

  @Override
  protected Content store(final Context context, final Content content) throws IOException {
    String path = getPath(context);
    AssetKind assetKind = getAssetKind(context);
    try (TempBlob tempBlob = content().getTempBlob(content)) {
      FluentAsset asset;
      switch (assetKind) {
        case CDN_METADATA:
          asset = content().putMetadata(path, tempBlob);
          break;
        case SPEC:
          asset = storeSpecFile(path, tempBlob);
          break;
        case POD:
          asset = storePodFile(context, path, tempBlob);
          break;
        default:
          throw new IllegalStateException("Received an invalid AssetKind of type: " + assetKind.name());
      }
      return asset.markAsCached(content).download();
    }
  }

  @Nullable
  @Override
  protected String getUrl(@Nonnull final Context context) {
    AssetKind assetKind = getAssetKind(context);
    switch (assetKind) {
      case CDN_METADATA:
      case SPEC:
        return getPath(context);
      case POD:
        return providePodExternalUri(context);
      default:
        throw new IllegalStateException("Received an invalid AssetKind of type: " + assetKind.name());
    }
  }

  @Nonnull
  @Override
  protected CacheController getCacheController(@Nonnull final Context context) {
    return cacheControllerHolder.require(getAssetKind(context).getCacheType());
  }

  private FluentAsset storeSpecFile(final String path, final TempBlob tempBlob) throws IOException {
    SpecInfo specInfo = null;
    try (InputStream in = tempBlob.get()) {
      specInfo = specFileProcessor.parse(new String(in.readAllBytes(), StandardCharsets.UTF_8));
    }
    catch (InvalidSpecFileException e) {
      log.debug("Spec file {} of {} is served as is: {}", path, getRepository().getName(), e.getMessage());
    }
    if (specInfo == null) {
      return content().putSpec(path, tempBlob, null, null, null);
    }
    URI remoteUri = specInfo.getRemoteUri();
    return content().putSpec(path, tempBlob, specInfo.getName(), specInfo.getVersion(),
        remoteUri == null ? null : remoteUri.toString());
  }

  private FluentAsset storePodFile(final Context context, final String path, final TempBlob tempBlob) {
    TokenMatcher.State state = context.getAttributes().require(TokenMatcher.State.class);
    String name = state.getTokens().get(PACKAGE_NAME_KEY);
    String version = state.getTokens().get(PACKAGE_VERSION_KEY);
    String remoteUrl = content().getPodRemoteUrl(PathUtils.buildSpecFilePath(name, version));
    return content().putPod(path, name, version, Strings.nullToEmpty(remoteUrl), tempBlob);
  }

  private Content transformSpecFile(final Content content) throws IOException {
    Asset asset = content.getAttributes().get(Asset.class);
    if (asset == null || Strings.isNullOrEmpty(
        asset.attributes(CocoapodsFormat.NAME).get(CocoapodsContentFacet.POD_REMOTE_ATTRIBUTE, String.class))) {
      return content;
    }
    String specFile;
    try (InputStream in = content.openInputStream()) {
      specFile = new String(in.readAllBytes(), StandardCharsets.UTF_8);
    }
    try {
      String proxied = specFileProcessor.toProxiedSpec(specFile, URI.create(getRepository().getUrl() + "/"));
      return new Content(content, new BytesPayload(proxied.getBytes(StandardCharsets.UTF_8), APPLICATION_JSON));
    }
    catch (InvalidSpecFileException e) {
      log.info("Invalid spec file {} in {}", asset.path(), getRepository().getName(), e);
      return content;
    }
  }

  @Nullable
  private String providePodExternalUri(final Context context) {
    TokenMatcher.State state = context.getAttributes().require(TokenMatcher.State.class);
    String name = state.getTokens().get(PACKAGE_NAME_KEY);
    String version = state.getTokens().get(PACKAGE_VERSION_KEY);
    String filename = state.getTokens().get(FILENAME_KEY);
    if (!PathUtils.isValidSegment(name) || !PathUtils.isValidSegment(version)) {
      return null;
    }
    String specPath = PathUtils.buildSpecFilePath(name, version);
    String remoteUrl = content().getPodRemoteUrl(specPath);
    if (remoteUrl == null && content().getAsset(specPath).isEmpty()) {
      fetchSpecFile(context, specPath);
      remoteUrl = content().getPodRemoteUrl(specPath);
    }
    if (remoteUrl == null) {
      log.debug("No source to proxy for pod {} {} in {}", name, version, getRepository().getName());
      return null;
    }
    if (!PathUtils.podFileName(version, URI.create(remoteUrl)).equals(filename)) {
      log.debug("Pod {} {} in {} has no file {}", name, version, getRepository().getName(), filename);
      return null;
    }
    return remoteUrl;
  }

  private void fetchSpecFile(final Context podContext, final String specPath) {
    Request request = new Request.Builder()
        .action(GET)
        .path("/" + specPath)
        .build();
    Context specContext = new Context(podContext.getRepository(), request);
    specContext.getAttributes().backing().putAll(podContext.getAttributes().backing());
    specContext.getAttributes().set(AssetKind.class, SPEC);
    try (Content spec = super.get(specContext)) {
      if (spec == null) {
        log.debug("Spec file {} is not available in {}", specPath, getRepository().getName());
      }
    }
    catch (IOException e) {
      log.debug("Failed to fetch spec file {} for {}", specPath, getRepository().getName(), e);
    }
  }

  private static String getPath(final Context context) {
    return removeInitialSlashFromPath(context.getRequest().getPath());
  }

  private static AssetKind getAssetKind(final Context context) {
    return context.getAttributes().require(AssetKind.class);
  }

  private CocoapodsContentFacet content() {
    return facet(CocoapodsContentFacet.class);
  }
}
