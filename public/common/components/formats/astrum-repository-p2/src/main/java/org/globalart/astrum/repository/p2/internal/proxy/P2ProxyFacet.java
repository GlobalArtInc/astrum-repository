
package org.globalart.astrum.repository.p2.internal.proxy;

import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.net.URI;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;

import javax.annotation.Nonnull;
import javax.annotation.Nullable;

import org.globalart.astrum.repository.cache.CacheController;
import org.globalart.astrum.repository.content.facet.ContentProxyFacetSupport;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.p2.P2ContentFacet;
import org.globalart.astrum.repository.p2.internal.AssetKind;
import org.globalart.astrum.repository.p2.internal.P2Format;
import org.globalart.astrum.repository.p2.internal.metadata.CompositeRepositoryRewriter;
import org.globalart.astrum.repository.p2.internal.metadata.RemoveMirrorTransformer;
import org.globalart.astrum.repository.p2.internal.metadata.UriToSiteHashUtil;
import org.globalart.astrum.repository.p2.internal.metadata.XmlTransformer;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.matchers.token.TokenMatcher;
import org.globalart.astrum.repository.view.payloads.StreamPayload;

import com.google.common.reflect.TypeToken;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.p2.internal.util.P2PathUtils.matcherState;
import static org.globalart.astrum.repository.p2.internal.util.P2PathUtils.site;
import static org.globalart.astrum.repository.p2.internal.util.P2PathUtils.toP2Attributes;
import static org.globalart.astrum.repository.p2.internal.util.P2PathUtils.toP2AttributesBinary;

@Component
@Qualifier(P2Format.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class P2ProxyFacet
    extends ContentProxyFacetSupport
{
  public static final String REMOTE_URL = "remote_url";

  public static final String REMOTE_HASH = "remote_site_hash";

  public static final String MIRRORS_URL = "mirrors_url";

  public static final String CHILD_URLS = "child_urls";

  @Nullable
  @Override
  protected Content getCachedContent(final Context context) {
    return content().get(context.getRequest().getPath()).orElse(null);
  }

  @Nonnull
  @Override
  protected CacheController getCacheController(@Nonnull final Context context) {
    return cacheControllerHolder.require(assetKind(context).getCacheType());
  }

  @Override
  protected Content store(final Context context, final Content content) throws IOException {
    AssetKind assetKind = assetKind(context);
    String remoteUrl = context.getAttributes().require(REMOTE_URL, String.class);
    TokenMatcher.State matcherState = matcherState(context);
    String path = context.getRequest().getPath();
    switch (assetKind) {
      case COMPOSITE_ARTIFACTS:
      case COMPOSITE_CONTENT:
        return storeCompositeMetadata(path, content, assetKind, matcherState, remoteUrl);
      case ARTIFACTS_METADATA:
        return storeArtifactsMetadata(path, content, matcherState, remoteUrl);
      case CONTENT_METADATA:
      case P2_INDEX:
        return content().putMetadata(path, assetKind, content, attributes(matcherState, remoteUrl));
      case BUNDLE:
        return content().putBundle(toP2Attributes(path, matcherState), content);
      case BINARY_BUNDLE:
        return content().putBinary(toP2AttributesBinary(path, matcherState), content);
      default:
        throw new IllegalStateException("Unexpected asset kind " + assetKind);
    }
  }

  @Nullable
  @Override
  protected String getUrl(@Nonnull final Context context) {
    TokenMatcher.State matcherState = matcherState(context);
    String site = site(matcherState);

    String remoteUrl;
    if (site == null) {
      String repositoryUrl = getRemoteUrl().toString();
      remoteUrl = URI.create(repositoryUrl.endsWith("/") ? repositoryUrl : repositoryUrl + '/')
          .resolve(context.getRequest().getPath().substring(1)).toString();
    }
    else {
      String path = context.getRequest().getPath().substring(2 + site.length());
      remoteUrl = findRepositoryUrl(site).map(base -> base.resolve(path)).map(URI::toString).orElse(null);
    }

    if (remoteUrl != null) {
      context.getAttributes().set(REMOTE_URL, remoteUrl);
    }
    return remoteUrl;
  }

  private Optional<URI> findRepositoryUrl(final String site) {
    for (FluentAsset asset : content().compositeMetadata()) {
      for (String url : childUrls(asset)) {
        if (UriToSiteHashUtil.map(url).equals(site)) {
          return Optional.of(URI.create(url));
        }
      }
    }
    log.debug("Unknown remote site {} in p2 proxy repository {}", site, getRepository().getName());
    return Optional.empty();
  }

  private static List<String> childUrls(final FluentAsset asset) {
    List<String> urls = asset.attributes(P2Format.NAME).get(CHILD_URLS, new TypeToken<List<String>>() { });
    return urls != null ? urls : List.of();
  }

  private Content storeCompositeMetadata(
      final String assetPath,
      final Content content,
      final AssetKind assetKind,
      final TokenMatcher.State matcherState,
      final String remoteUrl) throws IOException
  {
    String filename = AssetKind.COMPOSITE_ARTIFACTS == assetKind ? "compositeArtifacts.xml" : "compositeContent.xml";
    CompositeRepositoryRewriter rewriter = new CompositeRepositoryRewriter(URI.create(remoteUrl), site(matcherState) == null);

    try (StreamPayload payload = rewriteMetadata(assetPath, content, filename, rewriter)) {
      Map<String, Object> attributes = attributes(matcherState, remoteUrl);
      attributes.put(CHILD_URLS, rewriter.getUrls());
      return content().putMetadata(assetPath, assetKind, toContent(payload, content), attributes);
    }
  }

  private Content storeArtifactsMetadata(
      final String assetPath,
      final Content content,
      final TokenMatcher.State matcherState,
      final String remoteUrl) throws IOException
  {
    RemoveMirrorTransformer transformer = new RemoveMirrorTransformer();

    try (StreamPayload payload = rewriteMetadata(assetPath, content, "artifacts.xml", transformer)) {
      Map<String, Object> attributes = attributes(matcherState, remoteUrl);
      transformer.getMirrorsUrl().ifPresent(mirrorsUrl -> attributes.put(MIRRORS_URL, mirrorsUrl));
      return content().putMetadata(assetPath, AssetKind.ARTIFACTS_METADATA, toContent(payload, content), attributes);
    }
  }

  private StreamPayload rewriteMetadata(
      final String assetPath,
      final Content content,
      final String internalFilename,
      final XmlTransformer transformer) throws IOException
  {
    ByteArrayOutputStream buffer = new ByteArrayOutputStream();
    try (InputStream in = content.openInputStream()) {
      StreamCopier.copierFor(assetPath, internalFilename, in, buffer).process(transformer);
    }
    byte[] bytes = buffer.toByteArray();
    return new StreamPayload(() -> new ByteArrayInputStream(bytes), bytes.length, contentType(assetPath, content));
  }

  private static String contentType(final String assetPath, final Content content) {
    if (assetPath.endsWith(".jar")) {
      return "application/java-archive";
    }
    if (assetPath.endsWith(".xz")) {
      return "application/x-xz";
    }
    if (assetPath.endsWith(".xml")) {
      return "application/xml";
    }
    return content.getContentType();
  }

  private static Map<String, Object> attributes(final TokenMatcher.State matcherState, final String remoteUrl) {
    Map<String, Object> attributes = new HashMap<>();
    attributes.put(REMOTE_URL, remoteUrl);
    String site = site(matcherState);
    if (site != null) {
      attributes.put(REMOTE_HASH, site);
    }
    return attributes;
  }

  private static AssetKind assetKind(final Context context) {
    return context.getAttributes().require(AssetKind.class);
  }

  private P2ContentFacet content() {
    return getRepository().facet(P2ContentFacet.class);
  }

  private static Content toContent(final Payload payload, final Content originalContent) {
    Content streamContent = new Content(payload);
    streamContent.getAttributes().backing().putAll(originalContent.getAttributes().backing());
    return streamContent;
  }
}
