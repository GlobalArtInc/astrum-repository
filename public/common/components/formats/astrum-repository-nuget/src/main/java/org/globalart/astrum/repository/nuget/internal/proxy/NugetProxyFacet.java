package org.globalart.astrum.repository.nuget.internal.proxy;

import java.io.IOException;
import java.io.InputStream;
import java.net.URI;
import java.net.URISyntaxException;
import java.net.URLEncoder;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.Optional;
import java.util.Set;
import java.util.concurrent.TimeUnit;

import javax.annotation.Nonnull;
import javax.annotation.Nullable;

import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.cache.CacheController;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.config.ConfigurationFacet;
import org.globalart.astrum.repository.content.facet.ContentProxyFacetSupport;
import org.globalart.astrum.repository.http.HttpMethods;
import org.globalart.astrum.repository.nuget.internal.AssetKind;
import org.globalart.astrum.repository.nuget.internal.NugetPaths;
import org.globalart.astrum.repository.nuget.internal.content.NugetContentFacet;
import org.globalart.astrum.repository.nuget.internal.metadata.NugetV3Documents;
import org.globalart.astrum.repository.nuget.internal.metadata.NugetVersion;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Parameters;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.repository.view.payloads.StringPayload;

import com.google.common.cache.Cache;
import com.google.common.cache.CacheBuilder;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.nuget.internal.NugetPaths.lower;

@Facet.Exposed
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class NugetProxyFacet
    extends ContentProxyFacetSupport
{
  public static final String CONFIG_KEY = "nugetProxy";

  public static final String V2 = "V2";

  public static final String V3 = "V3";

  private static final String[] REGISTRATION_TYPES = {
      "RegistrationsBaseUrl/3.6.0", "RegistrationsBaseUrl/3.4.0", "RegistrationsBaseUrl/Versioned",
      "RegistrationsBaseUrl/3.0.0-rc", "RegistrationsBaseUrl/3.0.0-beta", "RegistrationsBaseUrl"
  };

  private static final String[] SEARCH_TYPES = {
      "SearchQueryService/3.5.0", "SearchQueryService/3.0.0-rc", "SearchQueryService/3.0.0-beta",
      "SearchQueryService"
  };

  private static final String[] AUTOCOMPLETE_TYPES = {
      "SearchAutocompleteService/3.5.0", "SearchAutocompleteService/3.0.0-rc",
      "SearchAutocompleteService/3.0.0-beta", "SearchAutocompleteService"
  };

  public static class Config
  {
    public Integer queryCacheItemMaxAge = 3600;

    public String nugetVersion = V3;
  }

  public record Upstream(
      String flatContainer,
      String registration,
      Set<String> registrations,
      @Nullable String search,
      @Nullable String autocomplete)
  {
  }

  private Config nugetConfig = new Config();

  private Cache<String, String> queryCache = CacheBuilder.newBuilder().maximumSize(0).build();

  @Override
  protected void doConfigure(final Configuration configuration) throws Exception {
    super.doConfigure(configuration);
    Config config = facet(ConfigurationFacet.class).readSection(configuration, CONFIG_KEY, Config.class);
    nugetConfig = config != null ? config : new Config();
    int maxAge = nugetConfig.queryCacheItemMaxAge == null ? 3600 : nugetConfig.queryCacheItemMaxAge;
    queryCache = maxAge > 0
        ? CacheBuilder.newBuilder().maximumSize(2000).expireAfterWrite(maxAge, TimeUnit.SECONDS).build()
        : CacheBuilder.newBuilder().maximumSize(0).build();
  }

  @Override
  protected URI normalizeURLPath(final URI remoteURI) {
    String path = remoteURI.getPath();
    if (path != null && path.toLowerCase(Locale.ROOT).endsWith(".json")) {
      return remoteURI;
    }
    return super.normalizeURLPath(remoteURI);
  }

  public boolean isV2() {
    return V2.equalsIgnoreCase(nugetConfig.nugetVersion);
  }

  private String remote() {
    return getRemoteUrl().toString();
  }

  private String serviceIndexUrl() {
    String remote = remote();
    if (remote.toLowerCase(Locale.ROOT).endsWith(".json")) {
      return remote;
    }
    return (remote.endsWith("/") ? remote : remote + "/") + "index.json";
  }

  private NugetContentFacet content() {
    return getRepository().facet(NugetContentFacet.class);
  }

  private static AssetKind kind(final Context context) {
    return context.getAttributes().require(AssetKind.class);
  }

  @Nullable
  @Override
  protected Content getCachedContent(final Context context) throws IOException {
    Optional<Content> content = content().get(assetPath(context));
    if (content.isEmpty()) {
      return null;
    }
    return kind(context) == AssetKind.REGISTRATION ? rewrite(content.get()) : content.get();
  }

  @Override
  protected Content store(final Context context, final Content content) throws IOException {
    AssetKind kind = kind(context);
    String path = assetPath(context);
    if (kind == AssetKind.PACKAGE) {
      return content().putPackage(path, content);
    }
    Content stored = content().putMetadata(path, content, kind);
    return kind == AssetKind.REGISTRATION ? rewrite(stored) : stored;
  }

  @Nonnull
  @Override
  protected CacheController getCacheController(@Nonnull final Context context) {
    return cacheControllerHolder.require(kind(context).getCacheType());
  }

  private String assetPath(final Context context) {
    AssetKind kind = kind(context);
    String path = context.getRequest().getPath();
    if (kind == AssetKind.SERVICE_INDEX) {
      return NugetPaths.UPSTREAM_SERVICE_INDEX;
    }
    if (kind == AssetKind.PACKAGE) {
      Optional<String[]> coordinates = NugetPaths.packageCoordinates(path);
      if (coordinates.isPresent()) {
        String version = coordinates.get()[1];
        return NugetPaths.packagePath(coordinates.get()[0], NugetVersion.normalize(version).orElse(version));
      }
    }
    return lower(path);
  }

  @Nullable
  @Override
  protected String getUrl(@Nonnull final Context context) {
    AssetKind kind = kind(context);
    String path = context.getRequest().getPath();
    if (kind == AssetKind.SERVICE_INDEX) {
      return serviceIndexUrl();
    }
    Optional<String[]> coordinates = kind == AssetKind.PACKAGE
        ? NugetPaths.packageCoordinates(path)
        : Optional.empty();
    if (isV2()) {
      return coordinates.map(value -> "package/" + value[0] + "/" + value[1]).orElse(null);
    }
    try {
      Upstream upstream = upstream(context);
      if (coordinates.isPresent()) {
        String id = coordinates.get()[0];
        String version = NugetVersion.normalize(coordinates.get()[1]).orElse(coordinates.get()[1]);
        return upstream.flatContainer() + lower(id) + "/" + lower(version) + "/"
            + NugetPaths.packageFileName(id, version);
      }
      if (path.startsWith(NugetPaths.CONTENT)) {
        return upstream.flatContainer() + lower(path.substring(NugetPaths.CONTENT.length()));
      }
      if (path.startsWith(NugetPaths.REGISTRATION)) {
        return upstream.registration() + lower(path.substring(NugetPaths.REGISTRATION.length()));
      }
    }
    catch (IOException e) {
      log.warn("Unable to read the upstream service index of {}: {}", getRepository().getName(), e.getMessage());
    }
    return null;
  }

  public Upstream upstream(final Context context) throws IOException {
    Request request = new Request.Builder().action(HttpMethods.GET).path(NugetPaths.UPSTREAM_SERVICE_INDEX).build();
    Context indexContext = new Context(getRepository(), request);
    indexContext.getAttributes().backing().putAll(context.getAttributes().backing());
    indexContext.getAttributes().set(AssetKind.class, AssetKind.SERVICE_INDEX);
    Content index = get(indexContext);
    if (index == null) {
      throw new IOException("The upstream service index is not available");
    }
    try (InputStream in = index.openInputStream()) {
      return parseUpstream(NugetV3Documents.read(new String(in.readAllBytes(), StandardCharsets.UTF_8)));
    }
  }

  @SuppressWarnings("unchecked")
  private static Upstream parseUpstream(final Map<String, Object> index) throws IOException {
    Object resourcesObject = index.get("resources");
    if (!(resourcesObject instanceof List<?> resources)) {
      throw new IOException("The upstream service index has no resources");
    }
    String flat = null;
    Map<String, String> byType = new java.util.HashMap<>();
    Set<String> registrations = new LinkedHashSet<>();
    for (Object resourceObject : resources) {
      if (!(resourceObject instanceof Map<?, ?> resource) || !(resource.get("@id") instanceof String id)) {
        continue;
      }
      for (String type : types(resource.get("@type"))) {
        byType.putIfAbsent(type, id);
        if (type.startsWith("PackageBaseAddress") && flat == null) {
          flat = id;
        }
        if (type.startsWith("RegistrationsBaseUrl")) {
          registrations.add(slash(id));
        }
      }
    }
    String registration = first(byType, REGISTRATION_TYPES);
    if (flat == null || registration == null) {
      throw new IOException("The upstream service index has no flat container or registration resource");
    }
    return new Upstream(slash(flat), slash(registration), registrations, first(byType, SEARCH_TYPES),
        first(byType, AUTOCOMPLETE_TYPES));
  }

  private static List<String> types(final Object type) {
    List<String> result = new ArrayList<>();
    if (type instanceof String string) {
      result.add(string);
    }
    else if (type instanceof List<?> list) {
      for (Object item : list) {
        if (item instanceof String string) {
          result.add(string);
        }
      }
    }
    return result;
  }

  @Nullable
  private static String first(final Map<String, String> byType, final String[] types) {
    for (String type : types) {
      if (byType.containsKey(type)) {
        return byType.get(type);
      }
    }
    return null;
  }

  private static String slash(final String url) {
    return url.endsWith("/") ? url : url + "/";
  }

  private Content rewrite(final Content content) throws IOException {
    String json;
    try (InputStream in = content.openInputStream()) {
      json = new String(in.readAllBytes(), StandardCharsets.UTF_8);
    }
    Upstream upstream = upstream(new Context(getRepository(), new Request.Builder().action(HttpMethods.GET)
        .path(NugetPaths.UPSTREAM_SERVICE_INDEX).build()));
    return new Content(content, new StringPayload(rewriteV3(json, upstream), StandardCharsets.UTF_8,
        NugetV3Documents.JSON));
  }

  private String rewriteV3(final String json, final Upstream upstream) {
    String repositoryUrl = getRepository().getUrl();
    List<String> registrations = new ArrayList<>(upstream.registrations());
    registrations.add(upstream.registration());
    registrations.sort(Comparator.comparingInt(String::length).reversed());
    String result = json;
    for (String registration : registrations) {
      result = result.replace(registration, repositoryUrl + NugetPaths.REGISTRATION);
    }
    return result.replace(upstream.flatContainer(), repositoryUrl + NugetPaths.CONTENT);
  }

  @Nullable
  public Content search(final Context context, final boolean autocomplete) throws IOException {
    Upstream upstream = upstream(context);
    String base = autocomplete ? upstream.autocomplete() : upstream.search();
    if (base == null) {
      return null;
    }
    String url = withQuery(base, context.getRequest().getParameters());
    String body = cachedText(url);
    if (body == null) {
      return null;
    }
    return new Content(new StringPayload(rewriteV3(body, upstream), StandardCharsets.UTF_8, NugetV3Documents.JSON));
  }

  @Nullable
  public Content passThroughV2(final Context context) throws IOException {
    String path = context.getRequest().getPath();
    String remote = remote().endsWith("/") ? remote() : remote() + "/";
    String url = withQuery(remote + path.substring(1), context.getRequest().getParameters());
    String body = cachedText(url);
    if (body == null) {
      return null;
    }
    String repositoryUrl = getRepository().getUrl();
    String rewritten = rewriteV2(body, remote, repositoryUrl);
    if (remote.startsWith("https://")) {
      rewritten = rewriteV2(rewritten, "http://" + remote.substring("https://".length()), repositoryUrl);
    }
    String contentType = rewritten.startsWith("<") || rewritten.startsWith("﻿<")
        ? (rewritten.contains("<feed") ? "application/atom+xml;type=feed;charset=utf-8" : "application/xml;charset=utf-8")
        : "text/plain;charset=utf-8";
    return new Content(new StringPayload(rewritten, StandardCharsets.UTF_8, contentType));
  }

  private static String rewriteV2(final String text, final String remoteWithSlash, final String repositoryUrl) {
    String remoteWithoutSlash = remoteWithSlash.substring(0, remoteWithSlash.length() - 1);
    return text.replace(remoteWithSlash, repositoryUrl + "/")
        .replace(remoteWithoutSlash + "\"", repositoryUrl + "\"")
        .replace(remoteWithoutSlash + "<", repositoryUrl + "<")
        .replace(remoteWithoutSlash + "'", repositoryUrl + "'");
  }

  @Nullable
  private String cachedText(final String url) throws IOException {
    String cached = queryCache.getIfPresent(url);
    if (cached != null) {
      return cached;
    }
    URI uri;
    try {
      uri = new URI(url);
    }
    catch (URISyntaxException e) {
      throw new IOException("Invalid upstream URL " + url, e);
    }
    Payload payload = getPayload(getRepository(), uri);
    if (payload == null) {
      return null;
    }
    String text;
    try (InputStream in = payload.openInputStream()) {
      text = new String(in.readAllBytes(), StandardCharsets.UTF_8);
    }
    finally {
      payload.close();
    }
    queryCache.put(url, text);
    return text;
  }

  private static String withQuery(final String base, final Parameters parameters) {
    StringBuilder query = new StringBuilder();
    for (Map.Entry<String, String> entry : parameters) {
      if (!query.isEmpty()) {
        query.append('&');
      }
      query.append(encode(entry.getKey()));
      if (entry.getValue() != null) {
        query.append('=').append(encode(entry.getValue()));
      }
    }
    if (query.isEmpty()) {
      return base;
    }
    return base + (base.contains("?") ? "&" : "?") + query;
  }

  private static String encode(final String value) {
    return URLEncoder.encode(value, StandardCharsets.UTF_8)
        .replace("+", "%20")
        .replace("%24", "$")
        .replace("%27", "'")
        .replace("%28", "(")
        .replace("%29", ")")
        .replace("%2C", ",")
        .replace("%3A", ":");
  }

  @Override
  public void invalidateProxyCaches() {
    super.invalidateProxyCaches();
    queryCache.invalidateAll();
  }
}
