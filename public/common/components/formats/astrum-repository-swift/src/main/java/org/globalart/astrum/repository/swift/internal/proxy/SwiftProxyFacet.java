package org.globalart.astrum.repository.swift.internal.proxy;

import java.io.IOException;
import java.io.InputStream;
import java.net.URI;
import java.net.URLEncoder;
import java.util.ArrayList;
import java.util.HashSet;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.Set;

import javax.annotation.Nonnull;
import javax.annotation.Nullable;

import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.cache.CacheController;
import org.globalart.astrum.repository.content.facet.ContentProxyFacetSupport;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.httpclient.HttpClientFacet;
import org.globalart.astrum.repository.swift.AssetKind;
import org.globalart.astrum.repository.swift.SwiftCoordinates;
import org.globalart.astrum.repository.swift.SwiftFormat;
import org.globalart.astrum.repository.swift.internal.Links;
import org.globalart.astrum.repository.swift.internal.Manifests;
import org.globalart.astrum.repository.swift.internal.ReleaseResponses;
import org.globalart.astrum.repository.swift.internal.SwiftContentFacet;
import org.globalart.astrum.repository.swift.internal.SwiftPaths;
import org.globalart.astrum.repository.swift.internal.SwiftResponses;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.payloads.BytesPayload;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.node.ObjectNode;
import org.apache.http.Header;
import org.apache.http.HttpResponse;
import org.apache.http.HttpStatus;
import org.apache.http.client.HttpClient;
import org.apache.http.client.methods.HttpGet;
import org.apache.http.client.methods.HttpRequestBase;
import org.apache.http.client.utils.HttpClientUtils;
import org.apache.http.util.EntityUtils;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static java.nio.charset.StandardCharsets.UTF_8;
import static org.globalart.astrum.repository.swift.internal.SwiftResponses.APPLICATION_JSON;
import static org.globalart.astrum.repository.swift.internal.SwiftResponses.LINK;

@Facet.Exposed
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
@Qualifier(SwiftFormat.NAME)
public class SwiftProxyFacet
    extends ContentProxyFacetSupport
{
  static final int MAX_PAGES = 100;

  static final int MAX_RELEASES_SIZE = 16 * 1024 * 1024;

  @Nullable
  @Override
  protected Content getCachedContent(final Context context) {
    return cachedAsset(context).map(FluentAsset::download).orElse(null);
  }

  Optional<FluentAsset> cachedAsset(final Context context) {
    SwiftCoordinates canonical = content().canonical(SwiftPaths.coordinates(context));
    return content().getAsset(assetPath(context, canonical));
  }

  static String assetPath(final Context context, final SwiftCoordinates canonical) {
    return switch (SwiftPaths.kind(context)) {
      case RELEASES -> canonical.getReleasesPath();
      case RELEASE -> canonical.getReleaseMetadataPath();
      case MANIFEST -> canonical.getManifestPath(SwiftPaths.manifestFilename(context));
      case ARCHIVE -> canonical.getArchivePath();
    };
  }

  @Nonnull
  @Override
  protected CacheController getCacheController(@Nonnull final Context context) {
    return cacheControllerHolder.require(SwiftPaths.kind(context).getCacheType());
  }

  @Override
  protected Content store(final Context context, final Content content) throws IOException {
    SwiftCoordinates coordinates = SwiftPaths.coordinates(context);
    FluentAsset asset = switch (SwiftPaths.kind(context)) {
      case RELEASES -> {
        Content merged = mergePages(context, content);
        yield content().storeReleases(coordinates, merged, locations(context));
      }
      case RELEASE -> content().storeRelease(coordinates, content);
      case MANIFEST -> {
        String filename = SwiftPaths.manifestFilename(context);
        Map<String, String> alternates = Manifests.MANIFEST.equals(filename) ? alternates(context) : Map.of();
        yield content().storeManifest(coordinates, filename, content, alternates);
      }
      case ARCHIVE -> content().storeArchive(coordinates, content);
    };
    return asset.markAsCached(content).download();
  }

  @Override
  protected String getUrl(@Nonnull final Context context) {
    SwiftCoordinates coordinates = SwiftPaths.coordinates(context);
    String path = coordinates.getScope() + "/" + coordinates.getName();
    return switch (SwiftPaths.kind(context)) {
      case RELEASES -> path;
      case RELEASE -> path + "/" + coordinates.getVersion();
      case MANIFEST -> {
        String swiftVersion = context.getAttributes().get(SwiftPaths.UPSTREAM_SWIFT_VERSION, String.class);
        yield path + "/" + coordinates.getVersion() + "/" + Manifests.MANIFEST
            + (swiftVersion == null ? "" : "?" + SwiftPaths.SWIFT_VERSION_PARAMETER + "=" + swiftVersion);
      }
      case ARCHIVE -> path + "/" + coordinates.getVersion() + SwiftCoordinates.ARCHIVE_EXTENSION;
    };
  }

  @Override
  protected HttpRequestBase buildFetchHttpRequest(final URI uri, final Context context) {
    HttpRequestBase request = super.buildFetchHttpRequest(uri, context);
    request.setHeader("Accept", accept(SwiftPaths.kind(context)));
    return request;
  }

  static String accept(final AssetKind kind) {
    String mediaType = switch (kind) {
      case MANIFEST -> "swift";
      case ARCHIVE -> "zip";
      default -> "json";
    };
    return "application/vnd.swift.registry.v1+" + mediaType;
  }

  public List<String> remoteIdentifiers(final String url) throws IOException {
    URI uri = getRemoteUrl().resolve("identifiers?url=" + URLEncoder.encode(url, UTF_8));
    JsonNode body = SwiftResponses.readQuietly(get(uri));
    List<String> identifiers = new ArrayList<>();
    JsonNode values = body == null ? null : body.get("identifiers");
    if (values != null && values.isArray()) {
      values.forEach(value -> {
        if (value.isTextual() && SwiftCoordinates.fromIdentifier(value.asText(), null) != null) {
          identifiers.add(value.asText());
        }
      });
    }
    return identifiers;
  }

  Content mergePages(final Context context, final Content content) throws IOException {
    byte[] first;
    try (InputStream in = content.openInputStream()) {
      first = in.readNBytes(MAX_RELEASES_SIZE + 1);
    }
    JsonNode body = SwiftResponses.readQuietly(first);
    if (first.length > MAX_RELEASES_SIZE || body == null || !(body.get(ReleaseResponses.RELEASES) instanceof ObjectNode)) {
      throw new IOException("Remote returned an invalid release list for " + SwiftPaths.coordinates(context));
    }
    ObjectNode releases = (ObjectNode) body.get(ReleaseResponses.RELEASES);
    URI current = getRemoteUrl().resolve(getUrl(context));
    String next = nextPage(responseLinks(context));
    Set<String> visited = new HashSet<>();
    for (int page = 1; next != null && page < MAX_PAGES && visited.add(next); page++) {
      current = current.resolve(next);
      if (!sameHost(current, getRemoteUrl())) {
        throw new IOException("Remote release list continues on another host: " + current);
      }
      HttpResponse response = execute(current);
      try {
        if (response.getStatusLine().getStatusCode() != HttpStatus.SC_OK || response.getEntity() == null) {
          throw new IOException("Remote returned " + response.getStatusLine() + " for " + current);
        }
        JsonNode pageBody = SwiftResponses.readQuietly(EntityUtils.toByteArray(response.getEntity()));
        JsonNode pageReleases = pageBody == null ? null : pageBody.get(ReleaseResponses.RELEASES);
        if (pageReleases instanceof ObjectNode object) {
          object.fields().forEachRemaining(entry -> {
            if (!releases.has(entry.getKey())) {
              releases.set(entry.getKey(), entry.getValue());
            }
          });
        }
        next = nextPage(headerValues(response.getHeaders(LINK)));
      }
      finally {
        HttpClientUtils.closeQuietly(response);
      }
    }
    ObjectNode merged = SwiftResponses.object();
    merged.set(ReleaseResponses.RELEASES, releases);
    return new Content(content, new BytesPayload(SwiftResponses.bytes(merged), APPLICATION_JSON));
  }

  static boolean sameHost(final URI uri, final URI remote) {
    return uri.getHost() != null && uri.getHost().equalsIgnoreCase(remote.getHost())
        && uri.getPort() == remote.getPort();
  }

  @Nullable
  static String nextPage(final List<String> linkHeaders) {
    return Links.parse(linkHeaders).stream()
        .filter(link -> "next".equals(link.rel()))
        .map(Links.Link::url)
        .findFirst()
        .orElse(null);
  }

  List<Map<String, String>> locations(final Context context) {
    List<Map<String, String>> locations = new ArrayList<>();
    for (Links.Link link : Links.parse(responseLinks(context))) {
      if (("canonical".equals(link.rel()) || "alternate".equals(link.rel())) && Links.isLinkable(link.url())) {
        locations.add(Map.of(ReleaseResponses.URL, link.url(), "rel", link.rel()));
      }
    }
    return locations;
  }

  static Map<String, String> alternates(final List<String> linkHeaders) {
    Map<String, String> alternates = new LinkedHashMap<>();
    for (Links.Link link : Links.parse(linkHeaders)) {
      String filename = link.param("filename");
      if ("alternate".equals(link.rel()) && Manifests.isVariant(filename)) {
        String toolsVersion = link.param("swift-tools-version");
        alternates.put(filename, Manifests.isSwiftVersion(toolsVersion) ? toolsVersion
            : Manifests.variantVersion(filename).orElse(""));
      }
    }
    return alternates;
  }

  private Map<String, String> alternates(final Context context) {
    return alternates(responseLinks(context));
  }

  private static List<String> responseLinks(final Context context) {
    HttpResponse response = context.getAttribute(HTTP_RESPONSE, HttpResponse.class);
    return response == null ? List.of() : headerValues(response.getHeaders(LINK));
  }

  private static List<String> headerValues(@Nullable final Header[] headers) {
    List<String> values = new ArrayList<>();
    if (headers != null) {
      for (Header header : headers) {
        values.add(header.getValue());
      }
    }
    return values;
  }

  @Nullable
  private byte[] get(final URI uri) throws IOException {
    HttpResponse response = execute(uri);
    try {
      if (response.getStatusLine().getStatusCode() != HttpStatus.SC_OK || response.getEntity() == null) {
        log.debug("Status {} fetching {}", response.getStatusLine(), uri);
        return null;
      }
      return EntityUtils.toByteArray(response.getEntity());
    }
    finally {
      HttpClientUtils.closeQuietly(response);
    }
  }

  private HttpResponse execute(final URI uri) throws IOException {
    HttpClient client = facet(HttpClientFacet.class).getHttpClient();
    HttpGet request = new HttpGet(uri);
    request.setHeader("Accept", accept(AssetKind.RELEASES));
    return client.execute(request);
  }

  private SwiftContentFacet content() {
    return facet(SwiftContentFacet.class);
  }
}
