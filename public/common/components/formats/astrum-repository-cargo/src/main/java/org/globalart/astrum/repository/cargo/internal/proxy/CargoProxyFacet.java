package org.globalart.astrum.repository.cargo.internal.proxy;

import java.io.IOException;
import java.io.InputStream;
import java.net.URI;
import java.net.URLEncoder;
import java.time.Duration;
import java.util.LinkedHashMap;
import java.util.Locale;
import java.util.Map;
import java.util.Optional;

import javax.annotation.Nonnull;
import javax.annotation.Nullable;

import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.cache.CacheController;
import org.globalart.astrum.repository.cargo.AssetKind;
import org.globalart.astrum.repository.cargo.CargoFormat;
import org.globalart.astrum.repository.cargo.CrateCoordinates;
import org.globalart.astrum.repository.cargo.internal.CargoContentFacet;
import org.globalart.astrum.repository.cargo.internal.CargoIndex;
import org.globalart.astrum.repository.cargo.internal.CargoPaths;
import org.globalart.astrum.repository.cargo.internal.CargoResponses;
import org.globalart.astrum.repository.content.facet.ContentProxyFacetSupport;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.httpclient.HttpClientFacet;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Context;

import com.fasterxml.jackson.databind.JsonNode;
import org.apache.http.HttpResponse;
import org.apache.http.HttpStatus;
import org.apache.http.client.HttpClient;
import org.apache.http.client.methods.HttpGet;
import org.apache.http.client.utils.HttpClientUtils;
import org.apache.http.util.EntityUtils;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static java.nio.charset.StandardCharsets.UTF_8;
import static org.globalart.astrum.repository.content.AttributeOperation.SET;

@Facet.Exposed
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
@Qualifier(CargoFormat.NAME)
public class CargoProxyFacet
    extends ContentProxyFacetSupport
{
  static final String REMOTE_CONFIG_ATTRIBUTE = "cargoRemoteConfig";

  private static final String DOWNLOAD_SUFFIX = "/{crate}/{version}/download";

  private static final long FAILURE_BACKOFF_MILLIS = 60_000L;

  private volatile RemoteConfig remoteConfig;

  private volatile long lastFailure;

  record RemoteConfig(String remoteUrl, String dl, String api, long fetched)
  {
  }

  @Override
  protected void doStart() throws Exception {
    super.doStart();
    remoteConfig = null;
    lastFailure = 0;
  }

  @Nullable
  @Override
  protected Content getCachedContent(final Context context) {
    return content().getAsset(assetPath(context)).map(FluentAsset::download).orElse(null);
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
    FluentAsset asset = switch (assetKind) {
      case INDEX -> content().saveIndex(assetPath(context), content);
      case CRATE -> content().saveCrate(CargoPaths.coordinates(context), content);
    };
    return asset.markAsCached(content).download();
  }

  @Override
  protected String getUrl(@Nonnull final Context context) {
    AssetKind assetKind = context.getAttributes().require(AssetKind.class);
    return switch (assetKind) {
      case INDEX -> CrateCoordinates.indexPath(CargoPaths.indexName(context));
      case CRATE -> downloadUrl(CargoPaths.coordinates(context));
    };
  }

  private static String assetPath(final Context context) {
    AssetKind assetKind = context.getAttributes().require(AssetKind.class);
    return switch (assetKind) {
      case INDEX -> "/" + CrateCoordinates.indexPath(CargoPaths.indexName(context));
      case CRATE -> CargoPaths.coordinates(context).getAssetPath();
    };
  }

  String downloadUrl(final CrateCoordinates coordinates) {
    RemoteConfig config = remoteConfig();
    String template = config.dl();
    if (!hasMarkers(template)) {
      template = stripTrailingSlash(template) + DOWNLOAD_SUFFIX;
    }
    String prefix = prefix(coordinates.getName());
    String url = template
        .replace("{crate}", coordinates.getName())
        .replace("{version}", coordinates.getVersion())
        .replace("{prefix}", prefix)
        .replace("{lowerprefix}", prefix.toLowerCase(Locale.ROOT))
        .replace("{sha256-checksum}", template.contains("{sha256-checksum}") ? checksum(coordinates) : "");
    return url.contains("://") ? url : getRemoteUrl().resolve(url).toString();
  }

  static boolean hasMarkers(final String template) {
    return template.contains("{crate}") || template.contains("{version}") || template.contains("{prefix}")
        || template.contains("{lowerprefix}") || template.contains("{sha256-checksum}");
  }

  static String prefix(final String name) {
    return switch (name.length()) {
      case 1 -> "1";
      case 2 -> "2";
      case 3 -> "3/" + name.charAt(0);
      default -> name.substring(0, 2) + "/" + name.substring(2, 4);
    };
  }

  private String checksum(final CrateCoordinates coordinates) {
    return content().getAsset("/" + coordinates.getIndexEntryPath())
        .map(FluentAsset::download)
        .map(CargoProxyFacet::readQuietly)
        .flatMap(bytes -> CargoIndex.lines(bytes).stream()
            .filter(line -> coordinates.getVersion().equals(CargoIndex.version(line)))
            .map(line -> CargoIndex.text(CargoResponses.read(line.getBytes(UTF_8)), "cksum"))
            .findFirst())
        .orElse("");
  }

  private static byte[] readQuietly(final Content content) {
    try (InputStream in = content.openInputStream()) {
      return in.readAllBytes();
    }
    catch (IOException e) {
      return new byte[0];
    }
  }

  public Optional<JsonNode> remoteSearch(final String query, final int perPage) throws IOException {
    String api = stripTrailingSlash(remoteConfig().api());
    URI uri = URI.create(api + "/api/v1/crates?q=" + URLEncoder.encode(query, UTF_8) + "&per_page=" + perPage);
    byte[] body = get(uri);
    return body == null ? Optional.empty() : Optional.of(CargoResponses.read(body));
  }

  RemoteConfig remoteConfig() {
    String remoteUrl = getRemoteUrl().toString();
    RemoteConfig current = remoteConfig;
    if (current != null && current.remoteUrl().equals(remoteUrl) && !expired(current)) {
      return current;
    }
    if (System.currentTimeMillis() - lastFailure < FAILURE_BACKOFF_MILLIS) {
      return fallback(current, remoteUrl);
    }
    try {
      byte[] body = get(getRemoteUrl().resolve("config.json"));
      if (body != null) {
        JsonNode config = CargoResponses.read(body);
        String dl = CargoIndex.text(config, "dl");
        String api = CargoIndex.text(config, "api");
        if (dl != null) {
          RemoteConfig fetched = new RemoteConfig(remoteUrl, dl, api != null ? api : remoteUrl,
              System.currentTimeMillis());
          remoteConfig = fetched;
          persist(fetched);
          return fetched;
        }
      }
      log.warn("Remote {} of repository {} has no usable config.json", remoteUrl, getRepository().getName());
    }
    catch (IOException | RuntimeException e) {
      log.warn("Unable to fetch config.json from {} for repository {}: {}", remoteUrl, getRepository().getName(),
          e.toString());
    }
    lastFailure = System.currentTimeMillis();
    return fallback(current, remoteUrl);
  }

  private RemoteConfig fallback(@Nullable final RemoteConfig current, final String remoteUrl) {
    if (current != null && current.remoteUrl().equals(remoteUrl)) {
      return current;
    }
    RemoteConfig persisted = persisted(remoteUrl);
    if (persisted != null) {
      remoteConfig = persisted;
      return persisted;
    }
    return new RemoteConfig(remoteUrl, stripTrailingSlash(remoteUrl) + CargoPaths.CRATES_API, remoteUrl, 0);
  }

  private boolean expired(final RemoteConfig config) {
    Duration maxAge = getConfiguration().getMetadataMaxAge();
    if (maxAge.isNegative()) {
      return false;
    }
    return System.currentTimeMillis() - config.fetched() > maxAge.toMillis();
  }

  private void persist(final RemoteConfig config) {
    try {
      Map<String, Object> value = new LinkedHashMap<>();
      value.put("remoteUrl", config.remoteUrl());
      value.put("dl", config.dl());
      value.put("api", config.api());
      content().attributes(SET, REMOTE_CONFIG_ATTRIBUTE, value);
    }
    catch (RuntimeException e) {
      log.debug("Unable to persist remote config of {}", getRepository().getName(), e);
    }
  }

  @Nullable
  private RemoteConfig persisted(final String remoteUrl) {
    try {
      Object value = content().attributes().get(REMOTE_CONFIG_ATTRIBUTE);
      if (value instanceof Map<?, ?> map && remoteUrl.equals(map.get("remoteUrl")) && map.get("dl") != null) {
        return new RemoteConfig(remoteUrl, String.valueOf(map.get("dl")),
            String.valueOf(map.get("api") != null ? map.get("api") : remoteUrl), 0);
      }
    }
    catch (RuntimeException e) {
      log.debug("Unable to read remote config of {}", getRepository().getName(), e);
    }
    return null;
  }

  @Nullable
  private byte[] get(final URI uri) throws IOException {
    HttpClient client = facet(HttpClientFacet.class).getHttpClient();
    HttpGet request = new HttpGet(uri);
    request.addHeader("Accept", "application/json");
    HttpResponse response = client.execute(request);
    try {
      int status = response.getStatusLine().getStatusCode();
      if (status != HttpStatus.SC_OK || response.getEntity() == null) {
        log.debug("Status {} fetching {}", status, uri);
        return null;
      }
      return EntityUtils.toByteArray(response.getEntity());
    }
    finally {
      HttpClientUtils.closeQuietly(response);
    }
  }

  private static String stripTrailingSlash(final String value) {
    return value.endsWith("/") ? value.substring(0, value.length() - 1) : value;
  }

  private CargoContentFacet content() {
    return facet(CargoContentFacet.class);
  }
}
