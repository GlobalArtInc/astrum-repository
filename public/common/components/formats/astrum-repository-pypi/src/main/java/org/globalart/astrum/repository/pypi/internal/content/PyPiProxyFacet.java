
package org.globalart.astrum.repository.pypi.internal.content;

import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.io.UncheckedIOException;
import java.net.URI;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;

import javax.annotation.Nonnull;
import javax.annotation.Nullable;

import org.globalart.astrum.common.template.TemplateHelper;
import org.globalart.astrum.repository.cache.CacheController;
import org.globalart.astrum.repository.content.facet.ContentProxyFacetSupport;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.pypi.content.PypiContentFacet;
import org.globalart.astrum.repository.pypi.internal.AssetKind;
import org.globalart.astrum.repository.pypi.internal.PyPiFileUtils;
import org.globalart.astrum.repository.pypi.internal.PyPiFormat;
import org.globalart.astrum.repository.pypi.internal.PyPiLink;
import org.globalart.astrum.repository.pypi.internal.PyPiPathUtils;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.ViewFacet;
import org.globalart.astrum.repository.view.payloads.BytesPayload;
import org.globalart.astrum.repository.view.payloads.TempBlob;

import com.google.common.base.Throwables;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.nio.charset.StandardCharsets.UTF_8;
import static java.util.Collections.singletonMap;
import static org.globalart.astrum.repository.http.HttpMethods.GET;
import static org.globalart.astrum.repository.pypi.internal.AssetKind.INDEX;
import static org.globalart.astrum.repository.pypi.internal.AssetKind.PACKAGE;
import static org.globalart.astrum.repository.pypi.internal.AssetKind.PACKAGE_SIGNATURE;
import static org.globalart.astrum.repository.pypi.internal.AssetKind.ROOT_INDEX;
import static org.globalart.astrum.repository.pypi.internal.PyPiAttributes.P_ASSET_KIND;
import static org.globalart.astrum.repository.pypi.internal.PyPiAttributes.P_NAME;
import static org.globalart.astrum.repository.pypi.internal.PyPiAttributes.P_SUMMARY;
import static org.globalart.astrum.repository.pypi.internal.PyPiAttributes.P_VERSION;
import static org.globalart.astrum.repository.pypi.internal.PyPiFileUtils.extractFilenameFromPath;
import static org.globalart.astrum.repository.pypi.internal.PyPiFileUtils.extractNameFromFilename;
import static org.globalart.astrum.repository.pypi.internal.PyPiFileUtils.extractVersionFromFilename;
import static org.globalart.astrum.repository.pypi.internal.PyPiIndexUtils.buildIndexPage;
import static org.globalart.astrum.repository.pypi.internal.PyPiIndexUtils.buildRootIndexPage;
import static org.globalart.astrum.repository.pypi.internal.PyPiIndexUtils.extractLinksFromIndex;
import static org.globalart.astrum.repository.pypi.internal.PyPiIndexUtils.makeIndexLinksAstrumPaths;
import static org.globalart.astrum.repository.pypi.internal.PyPiIndexUtils.makeRootIndexRelative;
import static org.globalart.astrum.repository.pypi.internal.PyPiIndexUtils.validateIndexLinks;
import static org.globalart.astrum.repository.pypi.internal.PyPiInfoUtils.extractMetadata;
import static org.globalart.astrum.repository.pypi.internal.PyPiPathUtils.matcherState;
import static org.globalart.astrum.repository.pypi.internal.PyPiPathUtils.name;
import static org.globalart.astrum.repository.pypi.internal.PyPiPathUtils.normalizeName;
import static org.globalart.astrum.repository.pypi.internal.PyPiPathUtils.path;
import static org.globalart.astrum.repository.pypi.internal.content.ContentPypiPathUtils.indexPath;
import static org.globalart.astrum.repository.pypi.internal.content.ContentPypiPathUtils.indexUriPath;
import static org.globalart.astrum.repository.pypi.internal.content.PyPiDataUtils.copySupportedAttributes;
import static org.globalart.astrum.repository.pypi.internal.content.PyPiDataUtils.setFormatAttribute;
import static org.globalart.astrum.repository.pypi.internal.content.PyPiDataUtils.setLastModified;
import static org.globalart.astrum.repository.view.ContentTypes.TEXT_HTML;

@Component
@Qualifier(PyPiFormat.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class PyPiProxyFacet
    extends ContentProxyFacetSupport
{
  private static final String SIGNATURE_EXTENSION = ".asc";

  private final TemplateHelper templateHelper;

  @Autowired
  public PyPiProxyFacet(final TemplateHelper templateHelper) {
    this.templateHelper = checkNotNull(templateHelper);
  }

  @Nullable
  @Override
  protected Content getCachedContent(final Context context) {
    switch (getAssetKind(context)) {
      case ROOT_INDEX:
        return content().getAsset(indexPath()).map(FluentAsset::download).orElse(null);
      case INDEX:
        return rewriteIndex(packageName(context));
      case PACKAGE:
      case PACKAGE_SIGNATURE:
        return content().getAsset(path(matcherState(context))).map(FluentAsset::download).orElse(null);
      default:
        throw new IllegalStateException();
    }
  }

  @Override
  @Nullable
  protected Content fetch(final Context context, final Content stale) throws IOException {
    try {
      return super.fetch(context, stale);
    }
    catch (NonResolvablePackageException e) {
      log.debug("Failed to resolve package {}", e.getMessage());
      return null;
    }
  }

  @Override
  protected Content store(final Context context, final Content content) throws IOException {
    switch (getAssetKind(context)) {
      case ROOT_INDEX:
        return putRootIndex(content);
      case INDEX:
        return putIndex(packageName(context), content);
      case PACKAGE:
      case PACKAGE_SIGNATURE:
        return putPackage(path(matcherState(context)), content);
      default:
        throw new IllegalStateException();
    }
  }

  @Override
  protected String getUrl(@Nonnull final Context context) {
    switch (getAssetKind(context)) {
      case ROOT_INDEX:
        return indexUriPath();
      case INDEX:
        return indexUriPath(packageName(context));
      default:
        return getPackageUrl(context);
    }
  }

  @Nonnull
  @Override
  protected CacheController getCacheController(@Nonnull final Context context) {
    return cacheControllerHolder.require(getAssetKind(context).getCacheType());
  }

  private String packageName(final Context context) {
    return normalizeName(name(matcherState(context)));
  }

  @Nullable
  private Content rewriteIndex(final String name) {
    Optional<FluentAsset> fluentAsset = content().getAsset(indexPath(name));
    if (fluentAsset.isEmpty()) {
      return null;
    }
    Content content = fluentAsset.get().download();
    try (InputStream inputStream = content.openInputStream()) {
      List<PyPiLink> links = makeIndexLinksAstrumPaths(name, inputStream);
      String html = buildIndexPage(templateHelper, name, links);
      return new Content(content, new BytesPayload(html.getBytes(UTF_8), TEXT_HTML));
    }
    catch (Exception e) {
      if (log.isDebugEnabled()) {
        log.warn("An error occurred re-writing the index for package {}", name, e);
      }
      else {
        log.warn("An error occurred re-writing the index for package {}: {}", name, e.toString());
      }
      return null;
    }
  }

  @Nullable
  private Content putIndex(final String name, final Content content) throws IOException {
    String html;
    try (InputStream inputStream = content.openInputStream()) {
      html = new String(inputStream.readAllBytes(), UTF_8);
    }
    if (!validateIndexLinks(name, extractLinksFromIndex(html))) {
      log.debug("Index of {} from {} does not reference any package of that name", name, getRepository().getName());
      return null;
    }
    storeHtmlPage(content, html, INDEX, indexPath(name));
    return rewriteIndex(name);
  }

  private Content putRootIndex(final Content content) throws IOException {
    String indexPage;
    try (InputStream inputStream = content.openInputStream()) {
      indexPage = buildRootIndexPage(templateHelper, makeRootIndexRelative(inputStream));
    }
    return storeHtmlPage(content, indexPage, ROOT_INDEX, indexPath());
  }

  private Content storeHtmlPage(
      final Content content,
      final String html,
      final AssetKind assetKind,
      final String path) throws IOException
  {
    try (ByteArrayInputStream stream = new ByteArrayInputStream(html.getBytes(UTF_8));
         TempBlob tempBlob = content().getTempBlob(stream, TEXT_HTML)) {
      setLastModified(content);
      return content().saveAsset(path, assetKind.name(), tempBlob)
          .withAttribute(PyPiFormat.NAME, singletonMap(P_ASSET_KIND, assetKind.name()))
          .markAsCached(content)
          .download();
    }
  }

  private Content putPackage(final String path, final Content content) throws IOException {
    String filename = extractFilenameFromPath(path);
    boolean signature = filename.endsWith(SIGNATURE_EXTENSION);
    String packageFilename = signature ? filename.substring(0, filename.length() - SIGNATURE_EXTENSION.length()) : filename;

    try (TempBlob tempBlob = content().getTempBlob(content)) {
      Map<String, String> attributes;
      if (signature) {
        attributes = new HashMap<>();
      }
      else {
        try (InputStream is = tempBlob.get()) {
          attributes = extractMetadata(is);
        }
      }
      if (!attributes.containsKey(P_NAME)) {
        attributes.put(P_NAME, extractNameFromFilename(packageFilename));
      }
      if (!attributes.containsKey(P_VERSION)) {
        attributes.put(P_VERSION, extractVersionFromFilename(packageFilename));
      }

      String name = checkNotNull(attributes.get(P_NAME));
      String version = checkNotNull(attributes.get(P_VERSION));
      FluentComponent component = content().findOrCreateComponent(name, version, normalizeName(name));
      String summary = attributes.get(P_SUMMARY);
      if (summary != null) {
        setFormatAttribute(component, P_SUMMARY, summary);
      }

      AssetKind assetKind = signature ? PACKAGE_SIGNATURE : PACKAGE;
      FluentAsset asset = content().saveAsset(path, component, assetKind.name(), tempBlob).markAsCached(content);
      copySupportedAttributes(asset, attributes);
      setFormatAttribute(asset, P_ASSET_KIND, assetKind.name());
      return asset.download();
    }
  }

  private String getPackageUrl(final Context context) {
    String requestPath = context.getRequest().getPath();
    String packageName = PyPiPathUtils.packageNameFromPath(requestPath);
    if (packageName == null) {
      throw new NonResolvablePackageException("Unable to determine package of " + requestPath);
    }
    packageName = normalizeName(packageName);
    String filename = PyPiFileUtils.extractFilenameFromPath(requestPath);
    boolean signature = filename.endsWith(SIGNATURE_EXTENSION);
    String packageFilename = signature ? filename.substring(0, filename.length() - SIGNATURE_EXTENSION.length()) : filename;

    String indexName = packageName;
    PyPiLink link = getExistingPackageLink(indexName, packageFilename)
        .orElseGet(() -> cachePackageIndexAndRetrieveLink(context, indexName, packageFilename));
    if (link == null) {
      throw new NonResolvablePackageException(
          "Unable to find reference for " + filename + " in package " + packageName);
    }

    URI remoteUrl = getRemoteUrl();
    if (!remoteUrl.getPath().endsWith("/")) {
      remoteUrl = URI.create(remoteUrl + "/");
    }
    String resolved = remoteUrl.resolve(indexUriPath(packageName)).resolve(stripFragment(link.getLink())).toString();
    return signature ? resolved + SIGNATURE_EXTENSION : resolved;
  }

  private static String stripFragment(final String link) {
    int hashIndex = link.indexOf('#');
    return hashIndex == -1 ? link : link.substring(0, hashIndex);
  }

  private Optional<PyPiLink> getExistingPackageLink(final String packageName, final String filename) {
    Optional<FluentAsset> fluentAsset = content().getAsset(indexPath(packageName));
    if (fluentAsset.isEmpty()) {
      return Optional.empty();
    }
    try (InputStream in = fluentAsset.get().download().openInputStream()) {
      return extractLinksFromIndex(in).stream().filter(link -> filename.equals(link.getFile())).findFirst();
    }
    catch (IOException e) {
      throw new UncheckedIOException(e);
    }
  }

  @Nullable
  private PyPiLink cachePackageIndexAndRetrieveLink(
      final Context context,
      final String packageName,
      final String filename)
  {
    try {
      Request request = new Request.Builder().action(GET).path('/' + indexUriPath(packageName)).build();
      Response response = getRepository().facet(ViewFacet.class).dispatch(request, context);
      if (!response.getStatus().isSuccessful()) {
        log.debug("Could not retrieve index of package {}", packageName);
      }
    }
    catch (IOException e) {
      throw new UncheckedIOException(e);
    }
    catch (Exception e) {
      Throwables.throwIfUnchecked(e);
      throw new UncheckedIOException(new IOException(e));
    }
    return getExistingPackageLink(packageName, filename).orElse(null);
  }

  private AssetKind getAssetKind(final Context context) {
    return context.getAttributes().require(AssetKind.class);
  }

  private PypiContentFacet content() {
    return getRepository().facet(PypiContentFacet.class);
  }
}
