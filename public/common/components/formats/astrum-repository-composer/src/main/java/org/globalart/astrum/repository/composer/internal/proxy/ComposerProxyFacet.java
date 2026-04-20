
package org.globalart.astrum.repository.composer.internal.proxy;

import java.io.IOException;
import java.io.UncheckedIOException;
import java.util.Map;

import javax.annotation.Nonnull;
import javax.annotation.Nullable;

import org.globalart.astrum.repository.cache.CacheController;
import org.globalart.astrum.repository.cache.CacheInfo;
import org.globalart.astrum.repository.composer.AssetKind;
import org.globalart.astrum.repository.composer.ComposerContentFacet;
import org.globalart.astrum.repository.composer.ComposerFormat;
import org.globalart.astrum.repository.composer.internal.ComposerJsonProcessor;
import org.globalart.astrum.repository.content.facet.ContentProxyFacetSupport;
import org.globalart.astrum.repository.http.HttpStatus;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.ViewFacet;
import org.globalart.astrum.repository.view.matchers.token.TokenMatcher;

import com.google.common.annotations.VisibleForTesting;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.composer.internal.ComposerPathUtils.buildPackagePath;
import static org.globalart.astrum.repository.composer.internal.ComposerPathUtils.buildPackagePathForDevVersions;
import static org.globalart.astrum.repository.composer.internal.ComposerPathUtils.buildProviderPath;
import static org.globalart.astrum.repository.composer.internal.ComposerPathUtils.buildZipballPath;
import static org.globalart.astrum.repository.composer.internal.recipe.ComposerRecipeSupport.PROJECT_TOKEN;
import static org.globalart.astrum.repository.composer.internal.recipe.ComposerRecipeSupport.VENDOR_TOKEN;
import static org.globalart.astrum.repository.composer.internal.recipe.ComposerRecipeSupport.VERSION_TOKEN;
import static org.globalart.astrum.repository.http.HttpMethods.GET;

@Component
@Qualifier(ComposerFormat.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class ComposerProxyFacet
    extends ContentProxyFacetSupport
{
  private static final String PACKAGES_JSON = "/packages.json";

  private static final String LIST_JSON = "/packages/list.json";

  private final ComposerJsonProcessor composerJsonProcessor;

  @Autowired
  public ComposerProxyFacet(final ComposerJsonProcessor composerJsonProcessor) {
    this.composerJsonProcessor = checkNotNull(composerJsonProcessor);
  }

  @Nullable
  @Override
  protected Content fetch(final Context context, final Content stale) throws IOException {
    try {
      return super.fetch(context, stale);
    }
    catch (NonResolvableProviderJsonException e) {
      log.debug("Composer dist URL not resolvable: {}", e.getMessage());
      return null;
    }
  }

  @Nullable
  @Override
  protected Content getCachedContent(final Context context) throws IOException {
    return content().get(assetPath(context)).orElse(null);
  }

  @Override
  protected Content store(final Context context, final Content content) throws IOException {
    AssetKind assetKind = context.getAttributes().require(AssetKind.class);
    Content toStore = assetKind == AssetKind.PACKAGES ? rewritePackagesJson(content) : content;
    return content().put(assetPath(context), toStore, assetKind);
  }

  @Override
  protected void indicateVerified(final Context context, final Content content, final CacheInfo cacheInfo)
      throws IOException
  {
    content().setCacheInfo(assetPath(context), content, cacheInfo);
  }

  @Override
  protected String getUrl(@Nonnull final Context context) {
    AssetKind assetKind = context.getAttributes().require(AssetKind.class);
    if (assetKind == AssetKind.ZIPBALL) {
      return getZipballUrl(context);
    }
    return context.getRequest().getPath().substring(1);
  }

  @Nonnull
  @Override
  protected CacheController getCacheController(@Nonnull final Context context) {
    AssetKind assetKind = context.getAttributes().require(AssetKind.class);
    return cacheControllerHolder.require(assetKind.getCacheType());
  }

  private String assetPath(final Context context) {
    AssetKind assetKind = context.getAttributes().require(AssetKind.class);
    switch (assetKind) {
      case PACKAGES:
        return PACKAGES_JSON;
      case LIST:
        return LIST_JSON;
      case PROVIDER:
        return buildProviderPath(context);
      case PACKAGE:
        return buildPackagePath(context);
      case ZIPBALL:
        return buildZipballPath(context);
      default:
        throw new IllegalStateException("Unexpected asset kind: " + assetKind);
    }
  }

  private Content rewritePackagesJson(final Content original) {
    try {
      Payload rewritten = composerJsonProcessor.rewritePackagesJson(getRepository(), original.getPayload());
      return new Content(original, rewritten);
    }
    catch (IOException e) {
      throw new UncheckedIOException(e);
    }
  }

  private String getZipballUrl(final Context context) {
    Map<String, String> tokens = context.getAttributes().require(TokenMatcher.State.class).getTokens();
    String vendor = tokens.get(VENDOR_TOKEN);
    String project = tokens.get(PROJECT_TOKEN);
    String version = tokens.get(VERSION_TOKEN);

    for (String path : new String[]{buildPackagePath(vendor, project),
        buildPackagePathForDevVersions(vendor, project)}) {
      try {
        Payload payload = getMetadataPayload(context, path);
        if (payload != null) {
          return composerJsonProcessor.getDistUrlFromPackage(vendor, project, version, payload);
        }
      }
      catch (Exception e) {
        log.debug("No dist URL for {}/{} {} in {}: {}", vendor, project, version, path, e.toString());
      }
    }

    try {
      Payload payload = getMetadataPayload(context, buildProviderPath(vendor, project));
      if (payload != null) {
        return composerJsonProcessor.getDistUrl(vendor, project, version, payload);
      }
    }
    catch (Exception e) {
      log.debug("No dist URL for {}/{} {} in provider metadata: {}", vendor, project, version, e.toString());
    }
    throw new NonResolvableProviderJsonException(
        String.format("No metadata found for vendor %s, project %s, version %s", vendor, project, version));
  }

  @Nullable
  private Payload getMetadataPayload(final Context context, final String path) throws Exception {
    Request request = new Request.Builder()
        .action(GET)
        .path(path)
        .attribute(ComposerProviderHandler.DO_NOT_REWRITE, "true")
        .build();
    Response response = getRepository().facet(ViewFacet.class).dispatch(request, context);
    if (response.getStatus().getCode() != HttpStatus.OK) {
      return null;
    }
    return response.getPayload();
  }

  private ComposerContentFacet content() {
    return getRepository().facet(ComposerContentFacet.class);
  }

  @VisibleForTesting
  static class NonResolvableProviderJsonException
      extends RuntimeException
  {
    NonResolvableProviderJsonException(final String message) {
      super(message);
    }
  }
}
