package org.globalart.astrum.repository.docker.internal.proxy;

import java.io.IOException;
import java.io.InputStream;
import java.net.URI;
import java.net.URISyntaxException;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.Objects;
import java.util.Set;
import java.util.concurrent.ConcurrentHashMap;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

import javax.annotation.Nonnull;
import javax.annotation.Nullable;

import org.globalart.astrum.repository.InvalidContentException;
import org.globalart.astrum.repository.cache.CacheController;
import org.globalart.astrum.repository.content.facet.ContentProxyFacetSupport;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.docker.DockerContentFacet;
import org.globalart.astrum.repository.docker.DockerFormat;
import org.globalart.astrum.repository.docker.internal.v2.DockerContentResponses;
import org.globalart.astrum.repository.docker.internal.v2.DockerDigest;
import org.globalart.astrum.repository.docker.internal.v2.DockerManifest;
import org.globalart.astrum.repository.docker.internal.v2.DockerManifest.InvalidManifestException;
import org.globalart.astrum.repository.docker.internal.v2.DockerResponses;
import org.globalart.astrum.repository.docker.internal.v2.DockerRoute;
import org.globalart.astrum.repository.httpclient.HttpClientFacet;
import org.globalart.astrum.repository.proxy.ProxyServiceException;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.payloads.TempBlob;
import org.globalart.astrum.validation.ssrf.AntiSsrfService;

import com.fasterxml.jackson.databind.JsonNode;
import org.apache.http.Header;
import org.apache.http.HttpHeaders;
import org.apache.http.HttpResponse;
import org.apache.http.client.HttpClient;
import org.apache.http.client.config.RequestConfig;
import org.apache.http.client.methods.Configurable;
import org.apache.http.client.methods.HttpGet;
import org.apache.http.client.methods.HttpRequestBase;
import org.apache.http.client.utils.HttpClientUtils;
import org.apache.http.client.utils.URIBuilder;
import org.apache.http.util.EntityUtils;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.common.hash.HashAlgorithm.SHA256;

@Component
@Qualifier(DockerFormat.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class DockerProxyFacet
    extends ContentProxyFacetSupport
{
  private static final CacheController IMMUTABLE = new CacheController(-1, null);

  private static final Set<String> DOCKER_HUB_HOSTS = Set.of("registry-1.docker.io", "index.docker.io", "docker.io",
      "registry.hub.docker.com");

  private static final Pattern CHALLENGE_PARAMETER = Pattern.compile("([a-zA-Z_]+)=\"([^\"]*)\"");

  private static final int MAX_REDIRECTS = 10;

  private static final long MIN_TOKEN_SECONDS = 30;

  private final Map<String, RemoteAuthorization> authorizations = new ConcurrentHashMap<>();

  private long maxManifestSize = 4L * 1024 * 1024;

  private AntiSsrfService dockerAntiSsrfService;

  @Autowired
  public void setMaxManifestSize(@Value("${astrum.docker.manifest.maxSize:4194304}") final long maxManifestSize) {
    this.maxManifestSize = maxManifestSize;
  }

  @Autowired(required = false)
  public void setDockerAntiSsrfService(@Nullable final AntiSsrfService antiSsrfService) {
    this.dockerAntiSsrfService = antiSsrfService;
  }

  private DockerContentFacet content() {
    return facet(DockerContentFacet.class);
  }

  @Override
  protected Content getCachedContent(final Context context) throws IOException {
    DockerRoute route = context.getAttributes().get(DockerRoute.class);
    if (route == null) {
      return null;
    }
    switch (route.getType()) {
      case MANIFEST:
        return content().findManifest(route.getName(), route.getReference()).map(FluentAsset::download).orElse(null);
      case BLOB:
        return content().findBlob(route.getReference()).map(FluentAsset::download).orElse(null);
      default:
        return null;
    }
  }

  @Nonnull
  @Override
  protected CacheController getCacheController(@Nonnull final Context context) {
    DockerRoute route = context.getAttributes().get(DockerRoute.class);
    if (route != null && (route.getType() == DockerRoute.Type.BLOB
        || (route.getType() == DockerRoute.Type.MANIFEST && route.isDigestReference()))) {
      return IMMUTABLE;
    }
    return cacheControllerHolder.getMetadataCacheController();
  }

  @Override
  protected String getUrl(@Nonnull final Context context) {
    DockerRoute route = context.getAttributes().get(DockerRoute.class);
    if (route == null || route.getName() == null) {
      return null;
    }
    String base = remoteBase() + "/v2/" + remoteName(route.getName());
    switch (route.getType()) {
      case MANIFEST:
        return base + "/manifests/" + route.getReference();
      case BLOB:
        return base + "/blobs/" + route.getReference();
      case TAGS:
        return base + "/tags/list";
      default:
        return null;
    }
  }

  @Override
  protected boolean shouldSkipStoreForHead() {
    return false;
  }

  @Override
  protected HttpRequestBase buildFetchHttpRequest(final URI uri, final Context context) {
    HttpGet request = new HttpGet(uri);
    DockerRoute route = context.getAttributes().get(DockerRoute.class);
    if (route != null && route.getType() == DockerRoute.Type.MANIFEST) {
      request.setHeader(HttpHeaders.ACCEPT, DockerManifest.ACCEPT);
    }
    else if (route != null && route.getType() == DockerRoute.Type.TAGS) {
      request.setHeader(HttpHeaders.ACCEPT, "application/json");
    }
    return request;
  }

  @Override
  protected HttpResponse execute(
      final Context context,
      final HttpClient client,
      final HttpRequestBase request) throws IOException
  {
    RequestConfig base = request.getConfig();
    if (base == null && client instanceof Configurable configurable) {
      base = configurable.getConfig();
    }
    RequestConfig config = RequestConfig.copy(base == null ? RequestConfig.DEFAULT : base)
        .setRedirectsEnabled(false)
        .build();
    request.setConfig(config);

    String scope = scope(context);
    applyAuthorization(request, scope);
    HttpResponse response = super.execute(context, client, request);
    if (response.getStatusLine().getStatusCode() == 401) {
      Header challenge = response.getFirstHeader(HttpHeaders.WWW_AUTHENTICATE);
      if (challenge != null) {
        HttpClientUtils.closeQuietly(response);
        if (authenticate(client, config, challenge.getValue(), scope)) {
          applyAuthorization(request, scope);
          response = super.execute(context, client, request);
        }
      }
    }

    HttpRequestBase current = request;
    int redirects = 0;
    while (isRedirect(response) && redirects < MAX_REDIRECTS) {
      Header location = response.getFirstHeader(HttpHeaders.LOCATION);
      if (location == null) {
        break;
      }
      URI target = current.getURI().resolve(location.getValue());
      if (dockerAntiSsrfService != null && target.getHost() != null) {
        dockerAntiSsrfService.validateHost(target.getHost());
      }
      HttpClientUtils.closeQuietly(response);
      HttpGet follow = new HttpGet(target);
      follow.setConfig(config);
      Header accept = request.getFirstHeader(HttpHeaders.ACCEPT);
      if (accept != null) {
        follow.setHeader(accept);
      }
      Header authorization = current.getFirstHeader(HttpHeaders.AUTHORIZATION);
      if (authorization != null && sameOrigin(current.getURI(), target)) {
        follow.setHeader(authorization);
      }
      log.debug("Following docker registry redirect {} -> {}", current.getURI(), target);
      current = follow;
      response = super.execute(context, client, follow);
      redirects++;
    }
    return response;
  }

  @Override
  protected void mayThrowProxyServiceException(final HttpResponse httpResponse) {
    int status = httpResponse.getStatusLine().getStatusCode();
    if (status == 401 || status == 403) {
      log.debug("Remote {} of {} denied access with {}, treating the content as missing", getRemoteUrl(),
          getRepository().getName(), status);
      return;
    }
    if (status == 429) {
      log.warn("Remote {} of {} is rate limiting requests", getRemoteUrl(), getRepository().getName());
      throw new ProxyServiceException(httpResponse);
    }
    super.mayThrowProxyServiceException(httpResponse);
  }

  private static boolean isRedirect(final HttpResponse response) {
    int status = response.getStatusLine().getStatusCode();
    return status == 301 || status == 302 || status == 303 || status == 307 || status == 308;
  }

  private static boolean sameOrigin(final URI source, final URI target) {
    return Objects.equals(source.getScheme(), target.getScheme())
        && Objects.equals(source.getHost(), target.getHost())
        && source.getPort() == target.getPort();
  }

  private String scope(final Context context) {
    DockerRoute route = context.getAttributes().get(DockerRoute.class);
    if (route == null || route.getName() == null) {
      return "registry:catalog:*";
    }
    return "repository:" + remoteName(route.getName()) + ":pull";
  }

  private void applyAuthorization(final HttpRequestBase request, final String scope) {
    RemoteAuthorization authorization = authorizations.get(scope);
    if (authorization != null && authorization.isExpired()) {
      authorizations.remove(scope, authorization);
      authorization = null;
    }
    request.removeHeaders(HttpHeaders.AUTHORIZATION);
    if (authorization != null) {
      request.setHeader(HttpHeaders.AUTHORIZATION, authorization.value);
    }
  }

  private boolean authenticate(
      final HttpClient client,
      final RequestConfig config,
      final String challenge,
      final String scope) throws IOException
  {
    String trimmed = challenge.trim();
    String scheme = trimmed.split("\\s+", 2)[0].toLowerCase(Locale.ENGLISH);
    Header basic = facet(HttpClientFacet.class).createBasicAuthHeader();
    if ("basic".equals(scheme)) {
      if (basic == null) {
        return false;
      }
      authorizations.put(scope, new RemoteAuthorization(basic.getValue(), Long.MAX_VALUE));
      return true;
    }
    if (!"bearer".equals(scheme)) {
      return false;
    }
    Map<String, String> parameters = new HashMap<>();
    Matcher matcher = CHALLENGE_PARAMETER.matcher(trimmed);
    while (matcher.find()) {
      parameters.put(matcher.group(1).toLowerCase(Locale.ENGLISH), matcher.group(2));
    }
    String realm = parameters.get("realm");
    if (realm == null || realm.isBlank()) {
      return false;
    }
    URI tokenUri;
    try {
      URIBuilder builder = new URIBuilder(realm);
      if (parameters.get("service") != null) {
        builder.addParameter("service", parameters.get("service"));
      }
      builder.addParameter("scope", parameters.getOrDefault("scope", scope));
      tokenUri = builder.build();
    }
    catch (URISyntaxException e) {
      log.warn("Invalid token realm {} from {}", realm, getRemoteUrl());
      return false;
    }
    if (dockerAntiSsrfService != null && tokenUri.getHost() != null) {
      dockerAntiSsrfService.validateHost(tokenUri.getHost());
    }
    HttpGet tokenRequest = new HttpGet(tokenUri);
    tokenRequest.setConfig(config);
    tokenRequest.setHeader(HttpHeaders.ACCEPT, "application/json");
    if (basic != null) {
      tokenRequest.setHeader(basic);
    }
    HttpResponse tokenResponse = client.execute(tokenRequest);
    try {
      int status = tokenResponse.getStatusLine().getStatusCode();
      if (status != 200 || tokenResponse.getEntity() == null) {
        log.warn("Token request to {} for {} failed with {}", tokenUri.getHost(), getRepository().getName(),
            tokenResponse.getStatusLine());
        return false;
      }
      JsonNode body = DockerResponses.MAPPER.readTree(EntityUtils.toByteArray(tokenResponse.getEntity()));
      String token = body.path("token").asText(null);
      if (token == null || token.isBlank()) {
        token = body.path("access_token").asText(null);
      }
      if (token == null || token.isBlank()) {
        return false;
      }
      long expiresIn = Math.max(body.path("expires_in").asLong(60) - 10, MIN_TOKEN_SECONDS);
      authorizations.put(scope, new RemoteAuthorization("Bearer " + token,
          System.currentTimeMillis() + expiresIn * 1000));
      return true;
    }
    finally {
      HttpClientUtils.closeQuietly(tokenResponse);
    }
  }

  @Override
  protected Content store(final Context context, final Content content) throws IOException {
    DockerRoute route = DockerRoute.get(context);
    switch (route.getType()) {
      case MANIFEST:
        return storeManifest(route, content);
      case BLOB:
        return storeBlob(route, content);
      default:
        throw new IllegalStateException("Unexpected docker proxy route " + route);
    }
  }

  private Content storeManifest(final DockerRoute route, final Content remote) throws IOException {
    byte[] bytes;
    try {
      bytes = DockerContentResponses.read(remote, maxManifestSize);
    }
    catch (DockerContentResponses.TooLargeException e) {
      throw new InvalidContentException("Remote manifest exceeds " + maxManifestSize + " bytes");
    }
    DockerManifest manifest;
    try {
      manifest = DockerManifest.parse(bytes, remote.getContentType());
    }
    catch (InvalidManifestException e) {
      throw new InvalidContentException("Invalid manifest from remote: " + e.getMessage());
    }
    String digest = DockerDigest.sha256(bytes);
    if (route.isDigestReference() && !manifest.isSchema1() && !digest.equals(route.getReference())) {
      throw new InvalidContentException("Remote manifest digest " + digest + " does not match " + route.getReference());
    }
    if (route.isDigestReference()) {
      digest = route.getReference();
    }
    DockerContentFacet content = content();
    FluentAsset asset = content.saveManifest(route.getName(), digest, bytes, manifest).markAsCached(remote);
    if (!route.isDigestReference()) {
      asset = content.saveTag(route.getName(), route.getReference(), digest, bytes, manifest).markAsCached(remote);
    }
    return asset.download();
  }

  private Content storeBlob(final DockerRoute route, final Content remote) {
    DockerContentFacet content = content();
    try (TempBlob blob = content.ingest(remote)) {
      String actual = DockerDigest.sha256(blob.getHashes().get(SHA256));
      if (!actual.equals(route.getReference())) {
        throw new InvalidContentException("Remote blob digest " + actual + " does not match " + route.getReference());
      }
      return content.saveBlob(blob, actual).markAsCached(remote).download();
    }
  }

  @Nullable
  public List<String> remoteTags(final Context context) throws IOException {
    DockerRoute route = DockerRoute.get(context);
    List<String> tags = new ArrayList<>();
    String url = getUrl(context);
    while (url != null) {
      Content page = fetch(url, context, null);
      if (page == null) {
        return tags.isEmpty() ? null : tags;
      }
      try (InputStream in = page.openInputStream()) {
        JsonNode body = DockerResponses.MAPPER.readTree(in);
        for (JsonNode tag : body.path("tags")) {
          if (tag.isTextual()) {
            tags.add(tag.asText());
          }
        }
      }
      finally {
        page.close();
      }
      url = nextLink(context, url);
      if (tags.size() > 100_000) {
        log.warn("Too many tags for {} in {}", route.getName(), getRepository().getName());
        break;
      }
    }
    return tags;
  }

  @Nullable
  private String nextLink(final Context context, final String current) {
    Object response = context.getAttributes().get(HTTP_RESPONSE);
    if (!(response instanceof HttpResponse httpResponse)) {
      return null;
    }
    Header link = httpResponse.getFirstHeader("Link");
    if (link == null) {
      return null;
    }
    String value = link.getValue();
    int start = value.indexOf('<');
    int end = value.indexOf('>');
    if (start < 0 || end <= start || !value.contains("rel=\"next\"")) {
      return null;
    }
    String next = URI.create(current).resolve(value.substring(start + 1, end)).toString();
    return next.equals(current) ? null : next;
  }

  private String remoteBase() {
    String url = getRemoteUrl().toString();
    while (url.endsWith("/")) {
      url = url.substring(0, url.length() - 1);
    }
    if (url.endsWith("/v2")) {
      url = url.substring(0, url.length() - 3);
    }
    return url;
  }

  public String remoteName(final String name) {
    String host = getRemoteUrl().getHost();
    if (host != null && DOCKER_HUB_HOSTS.contains(host.toLowerCase(Locale.ENGLISH)) && name.indexOf('/') < 0) {
      return "library/" + name;
    }
    return name;
  }

  private static final class RemoteAuthorization
  {
    private final String value;

    private final long expiresAt;

    private RemoteAuthorization(final String value, final long expiresAt) {
      this.value = value;
      this.expiresAt = expiresAt;
    }

    private boolean isExpired() {
      return System.currentTimeMillis() >= expiresAt;
    }
  }
}
