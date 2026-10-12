package org.globalart.astrum.repository.terraform.internal.proxy;

import java.io.IOException;
import java.io.InputStream;
import java.net.URI;
import java.util.Locale;
import java.util.Map;
import java.util.Optional;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeUnit;

import javax.annotation.Nonnull;
import javax.annotation.Nullable;

import org.globalart.astrum.common.hash.HashAlgorithm;
import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.cache.CacheController;
import org.globalart.astrum.repository.content.facet.ContentProxyFacetSupport;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.http.HttpMethods;
import org.globalart.astrum.repository.httpclient.HttpClientFacet;
import org.globalart.astrum.repository.terraform.AssetKind;
import org.globalart.astrum.repository.terraform.ModuleArchives;
import org.globalart.astrum.repository.terraform.ModuleCoordinates;
import org.globalart.astrum.repository.terraform.Platform;
import org.globalart.astrum.repository.terraform.ProviderCoordinates;
import org.globalart.astrum.repository.terraform.TerraformContentFacet;
import org.globalart.astrum.repository.terraform.TerraformFormat;
import org.globalart.astrum.repository.terraform.TerraformPaths;
import org.globalart.astrum.repository.terraform.TerraformRequest;
import org.globalart.astrum.repository.terraform.internal.ModulePackages;
import org.globalart.astrum.repository.terraform.internal.TerraformRepositoryConfig;
import org.globalart.astrum.repository.terraform.internal.TerraformResponses;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.repository.view.payloads.TempBlob;

import com.fasterxml.jackson.databind.JsonNode;
import com.google.common.cache.Cache;
import com.google.common.cache.CacheBuilder;
import org.apache.http.Header;
import org.apache.http.HttpResponse;
import org.apache.http.client.methods.HttpGet;
import org.apache.http.client.methods.HttpRequestBase;
import org.apache.http.client.utils.HttpClientUtils;
import org.apache.http.util.EntityUtils;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.terraform.TerraformContentFacet.ARCH;
import static org.globalart.astrum.repository.terraform.TerraformContentFacet.OS;
import static org.globalart.astrum.repository.terraform.TerraformContentFacet.SUBDIR;
import static org.globalart.astrum.repository.terraform.internal.TerraformResponses.X_TERRAFORM_GET;

/**
 * Proxies a registry that speaks the module and provider registry protocols, such as
 * {@code https://registry.terraform.io}.
 *
 * Registry responses are cached as metadata. Packages are cached under the repository: module packages at
 * {@code /v1/modules/<namespace>/<name>/<system>/<version>/} and provider release files at
 * {@code /v1/providers/<namespace>/<type>/<version>/}, fetched from the locations the upstream registry names,
 * and the repository hands out its own locations instead. Provider packages are checked against the upstream
 * checksum before they are stored.
 */
@Facet.Exposed
@Component
@Qualifier(TerraformFormat.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class TerraformProxyFacet
    extends ContentProxyFacetSupport
{
  static final String EXPECTED_SHA256 = TerraformProxyFacet.class.getName() + ".expectedSha256";

  private static final long FAILURE_BACKOFF_MILLIS = 60_000L;

  private final Cache<ModuleCoordinates, Optional<String>> moduleLocations = CacheBuilder.newBuilder()
      .maximumSize(2_000)
      .expireAfterWrite(10, TimeUnit.MINUTES)
      .build();

  private volatile Services services;

  private volatile long lastFailure;

  /**
   * The service URLs of the upstream registry, from its service discovery.
   */
  record Services(String remoteUrl, URI modules, URI providers, long fetched)
  {
  }

  @Override
  protected void doStart() throws Exception {
    super.doStart();
    services = null;
    lastFailure = 0;
    moduleLocations.invalidateAll();
  }

  @Override
  public void invalidateProxyCaches() {
    super.invalidateProxyCaches();
    services = null;
    moduleLocations.invalidateAll();
  }

  @Nullable
  @Override
  protected Content getCachedContent(final Context context) {
    return content().getAsset(TerraformRequest.of(context).assetPath()).map(FluentAsset::download).orElse(null);
  }

  @Nonnull
  @Override
  protected CacheController getCacheController(@Nonnull final Context context) {
    AssetKind assetKind = context.getAttributes().require(AssetKind.class);
    return cacheControllerHolder.require(assetKind.getCacheType());
  }

  @Override
  protected Content store(final Context context, final Content content) throws IOException {
    TerraformRequest request = TerraformRequest.of(context);
    AssetKind kind = context.getAttributes().require(AssetKind.class);
    TerraformContentFacet facet = content();
    try (TempBlob blob = facet.ingest(content)) {
      FluentAsset asset = switch (kind) {
        case MODULE_VERSIONS, PROVIDER_VERSIONS, PROVIDER_DOWNLOAD ->
            facet.putMetadata(request.assetPath(), kind, blob);
        case MODULE_ARCHIVE -> {
          ModuleSource source = context.getAttributes().get(ModuleSource.class);
          String extension = ModuleArchives.extension(request.file()).orElse(ModuleArchives.TAR_GZ);
          String subdir = ModulePackages.subdirectory(blob, extension, source == null ? null : source.subdir());
          yield facet.putModule(request.module(), request.assetPath(), blob, Map.of(SUBDIR, subdir));
        }
        case PROVIDER_ARCHIVE -> {
          String expected = context.getAttributes().get(EXPECTED_SHA256, String.class);
          String actual = blob.getHashes().get(HashAlgorithm.SHA256).toString();
          if (expected == null || !expected.equalsIgnoreCase(actual)) {
            throw new IOException("Provider package " + request.file() + " from " + getRemoteUrl()
                + " does not match the checksum its registry published");
          }
          Platform platform = request.platform();
          yield facet.putProviderFile(request.provider(), request.file(), kind, blob,
              Map.of(OS, platform.os(), ARCH, platform.arch()));
        }
        case PROVIDER_SHASUMS, PROVIDER_SIGNATURE ->
            facet.putProviderFile(request.provider(), request.file(), kind, blob, Map.of());
        default -> throw new IllegalStateException("Not a proxied asset kind: " + kind);
      };
      return asset.markAsCached(content).download();
    }
  }

  @Nullable
  @Override
  protected String getUrl(@Nonnull final Context context) {
    TerraformRequest request = TerraformRequest.of(context);
    AssetKind kind = context.getAttributes().require(AssetKind.class);
    try {
      return switch (kind) {
        case MODULE_VERSIONS -> services().modules().resolve(relativeModulePath(request.module()) + "versions")
            .toString();
        case PROVIDER_VERSIONS -> services().providers().resolve(relativeProviderPath(request.provider())
            + "versions").toString();
        case PROVIDER_DOWNLOAD -> downloadEndpoint(request.provider(), request.platform()).toString();
        case MODULE_ARCHIVE -> moduleArchiveUrl(context, request);
        case PROVIDER_ARCHIVE -> providerArchiveUrl(context, request);
        case PROVIDER_SHASUMS -> releaseFileUrl(request, "shasums_url");
        case PROVIDER_SIGNATURE -> releaseFileUrl(request, "shasums_signature_url");
        default -> null;
      };
    }
    catch (IOException e) {
      log.debug("Cannot locate {} upstream for repository {}: {}", request.assetPath(), getRepository().getName(),
          e.toString());
      return null;
    }
  }

  /**
   * Sends the bearer token of the repository to the upstream registry host only, never to the hosts packages are
   * downloaded from.
   */
  @Override
  protected HttpRequestBase buildFetchHttpRequest(final URI uri, final Context context) {
    HttpRequestBase request = super.buildFetchHttpRequest(uri, context);
    authorize(request, uri);
    return request;
  }

  /**
   * The host name providers of the upstream registry carry in their addresses, which the network mirror protocol
   * names in its paths: the configured upstream host, else the host of the remote URL.
   */
  public String upstreamHost() {
    Optional<String> configured = TerraformRepositoryConfig.upstreamHost(getRepository().getConfiguration());
    if (configured.isPresent()) {
      return configured.get();
    }
    URI remote = getRemoteUrl();
    String host = remote.getHost() == null ? "" : remote.getHost().toLowerCase(Locale.ROOT);
    int port = remote.getPort();
    boolean defaultPort = port < 0 || ("https".equalsIgnoreCase(remote.getScheme()) && port == 443)
        || ("http".equalsIgnoreCase(remote.getScheme()) && port == 80);
    return defaultPort ? host : host + ":" + port;
  }

  /**
   * The package location the upstream registry gives for a module version, resolved against its download endpoint.
   */
  public Optional<String> moduleLocation(final ModuleCoordinates module) throws IOException {
    try {
      return moduleLocations.get(module, () -> fetchModuleLocation(module));
    }
    catch (ExecutionException e) {
      if (e.getCause() instanceof IOException io) {
        throw io;
      }
      throw new IOException(e.getCause());
    }
  }

  /**
   * Caches the package of a module version, so its location can name the directory to read the module from.
   */
  public Optional<FluentAsset> cacheModule(final Context context, final ModuleCoordinates module,
                                           final ModuleSource source) throws IOException
  {
    TerraformRequest request = TerraformRequest.moduleArchive(module, module.archiveFileName(source.extension()));
    if (get(nested(context, request)) == null) {
      return Optional.empty();
    }
    return content().getAsset(request.assetPath());
  }

  /**
   * The cached or freshly fetched registry document of a request path.
   */
  public Optional<JsonNode> document(final Context context, final TerraformRequest request) throws IOException {
    Context nested = nested(context, request);
    Content content = get(nested);
    if (content == null) {
      return Optional.empty();
    }
    return Optional.ofNullable(TerraformResponses.read(content.getPayload()));
  }

  /**
   * The cached or freshly fetched bytes of a release file of a provider version.
   */
  public Optional<byte[]> releaseFile(final Context context, final ProviderCoordinates provider, final String file,
                                      final AssetKind kind) throws IOException
  {
    TerraformRequest request = TerraformRequest.providerFile(provider, file, kind);
    Content content = get(nested(context, request));
    if (content == null) {
      return Optional.empty();
    }
    try (InputStream in = content.openInputStream()) {
      return Optional.of(in.readAllBytes());
    }
  }

  public Optional<JsonNode> providerVersions(final Context context, final ProviderCoordinates provider)
      throws IOException
  {
    return document(context, TerraformRequest.providerVersions(provider));
  }

  public Optional<JsonNode> providerDownload(final Context context, final ProviderCoordinates provider,
                                             final Platform platform) throws IOException
  {
    return document(context, TerraformRequest.providerDownload(provider, platform));
  }

  /**
   * A package description of the provider version, for any platform: the checksums and the signature are the same
   * for all of them.
   */
  public Optional<JsonNode> anyProviderDownload(final Context context, final ProviderCoordinates provider)
      throws IOException
  {
    for (FluentAsset asset : content().assetsUnder(provider.directory() + "/download", AssetKind.PROVIDER_DOWNLOAD)) {
      try (InputStream in = asset.download().openInputStream()) {
        return Optional.of(TerraformResponses.read(in.readAllBytes()));
      }
      catch (IOException e) {
        log.debug("Unreadable {} in repository {}", asset.path(), getRepository().getName(), e);
      }
    }
    Optional<JsonNode> versions = providerVersions(context, provider);
    if (versions.isEmpty()) {
      return Optional.empty();
    }
    for (JsonNode entry : versions.get().path("versions")) {
      if (provider.version().equals(entry.path("version").asText())) {
        for (JsonNode platform : entry.path("platforms")) {
          String os = platform.path("os").asText();
          String arch = platform.path("arch").asText();
          if (TerraformPaths.isValidPlatformPart(os) && TerraformPaths.isValidPlatformPart(arch)) {
            Optional<JsonNode> download = providerDownload(context, provider, new Platform(os, arch));
            if (download.isPresent()) {
              return download;
            }
          }
        }
      }
    }
    return Optional.empty();
  }

  /**
   * The provider download endpoint of the upstream registry, which relative URLs in its answer are resolved
   * against.
   */
  URI downloadEndpoint(final ProviderCoordinates provider, final Platform platform) throws IOException {
    return services().providers().resolve(relativeProviderPath(provider) + provider.version() + "/download/"
        + platform.os() + "/" + platform.arch());
  }

  private String moduleArchiveUrl(final Context context, final TerraformRequest request) throws IOException {
    ModuleCoordinates module = request.module();
    Optional<String> location = moduleLocation(module);
    if (location.isEmpty()) {
      return null;
    }
    ModuleSource source = ModuleSource.parse(location.get());
    if (!source.cacheable() || !module.archiveFileName(source.extension()).equals(request.file())) {
      return null;
    }
    context.getAttributes().set(ModuleSource.class, source);
    return source.archiveUrl();
  }

  @Nullable
  private String providerArchiveUrl(final Context context, final TerraformRequest request) throws IOException {
    ProviderCoordinates provider = request.provider();
    Platform platform = request.platform();
    if (platform == null) {
      return null;
    }
    Optional<JsonNode> download = providerDownload(context, provider, platform);
    if (download.isEmpty() || !request.file().equals(download.get().path("filename").asText())) {
      return null;
    }
    String shasum = download.get().path("shasum").asText();
    if (shasum.isEmpty()) {
      return null;
    }
    context.getAttributes().set(EXPECTED_SHA256, shasum);
    return resolveUrl(downloadEndpoint(provider, platform), download.get().path("download_url").asText());
  }

  @Nullable
  private String releaseFileUrl(final TerraformRequest request, final String field) throws IOException {
    Context context = new Context(getRepository(), new Request.Builder().action(HttpMethods.GET)
        .path(request.assetPath()).build());
    Optional<JsonNode> download = anyProviderDownload(context, request.provider());
    if (download.isEmpty()) {
      return null;
    }
    String url = download.get().path(field).asText();
    if (url.isEmpty() || !request.file().equals(baseName(url))) {
      return null;
    }
    Platform platform = new Platform(download.get().path("os").asText(), download.get().path("arch").asText());
    return resolveUrl(downloadEndpoint(request.provider(), platform), url);
  }

  private Optional<String> fetchModuleLocation(final ModuleCoordinates module) throws IOException {
    URI endpoint = services().modules().resolve(relativeModulePath(module) + module.version() + "/download");
    HttpGet get = new HttpGet(endpoint);
    get.addHeader("Accept", "application/json");
    authorize(get, endpoint);
    HttpResponse response = facet(HttpClientFacet.class).getHttpClient().execute(get);
    try {
      int status = response.getStatusLine().getStatusCode();
      if (status == 404) {
        return Optional.empty();
      }
      if (status != 200 && status != 204) {
        throw new IOException("Upstream answered " + status + " for " + endpoint);
      }
      Header header = response.getFirstHeader(X_TERRAFORM_GET);
      String location = header == null ? null : header.getValue();
      if ((location == null || location.isBlank()) && response.getEntity() != null) {
        byte[] body = EntityUtils.toByteArray(response.getEntity());
        if (body.length > 0) {
          location = TerraformResponses.read(body).path("location").asText(null);
        }
      }
      if (location == null || location.isBlank()) {
        throw new IOException("Upstream gave no package location for " + endpoint);
      }
      return Optional.of(ModuleSource.resolve(endpoint, location));
    }
    finally {
      HttpClientUtils.closeQuietly(response);
    }
  }

  Services services() throws IOException {
    String remoteUrl = getRemoteUrl().toString();
    Services current = services;
    if (current != null && current.remoteUrl().equals(remoteUrl) && !expired(current)) {
      return current;
    }
    if (System.currentTimeMillis() - lastFailure < FAILURE_BACKOFF_MILLIS) {
      return fallback(current, remoteUrl);
    }
    URI discovery = base().resolve(".well-known/terraform.json");
    HttpGet get = new HttpGet(discovery);
    get.addHeader("Accept", "application/json");
    authorize(get, discovery);
    HttpResponse response = null;
    try {
      response = facet(HttpClientFacet.class).getHttpClient().execute(get);
      if (response.getStatusLine().getStatusCode() == 200 && response.getEntity() != null) {
        JsonNode document = TerraformResponses.read(EntityUtils.toByteArray(response.getEntity()));
        String modules = document.path(TerraformPaths.MODULES_SERVICE).asText(null);
        String providers = document.path(TerraformPaths.PROVIDERS_SERVICE).asText(null);
        Services discovered = new Services(remoteUrl,
            withSlash(modules == null ? base().resolve("v1/modules/") : discovery.resolve(modules)),
            withSlash(providers == null ? base().resolve("v1/providers/") : discovery.resolve(providers)),
            System.currentTimeMillis());
        services = discovered;
        return discovered;
      }
      log.warn("Remote {} of repository {} answered {} to service discovery", remoteUrl, getRepository().getName(),
          response.getStatusLine().getStatusCode());
    }
    catch (IOException | RuntimeException e) {
      log.warn("Service discovery of {} for repository {} failed: {}", remoteUrl, getRepository().getName(),
          e.toString());
    }
    finally {
      HttpClientUtils.closeQuietly(response);
    }
    lastFailure = System.currentTimeMillis();
    return fallback(current, remoteUrl);
  }

  private Services fallback(@Nullable final Services current, final String remoteUrl) {
    if (current != null && current.remoteUrl().equals(remoteUrl)) {
      return current;
    }
    return new Services(remoteUrl, base().resolve("v1/modules/"), base().resolve("v1/providers/"), 0);
  }

  private boolean expired(final Services current) {
    long maxAge = getConfiguration().getMetadataMaxAge().toMillis();
    return maxAge >= 0 && System.currentTimeMillis() - current.fetched() > maxAge;
  }

  private URI base() {
    String remote = getRemoteUrl().toString();
    return URI.create(remote.endsWith("/") ? remote : remote + "/");
  }

  private void authorize(final HttpRequestBase request, final URI uri) {
    String token = facet(HttpClientFacet.class).getBearerToken();
    if (token == null || token.isBlank() || uri.getHost() == null) {
      return;
    }
    String host = uri.getHost().toLowerCase(Locale.ROOT);
    boolean registryHost = host.equals(hostOf(getRemoteUrl()));
    Services known = services;
    if (known != null) {
      registryHost = registryHost || host.equals(hostOf(known.modules())) || host.equals(hostOf(known.providers()));
    }
    if (registryHost) {
      request.setHeader("Authorization", "Bearer " + token);
    }
  }

  private Context nested(final Context context, final TerraformRequest request) {
    Request nestedRequest = new Request.Builder()
        .action(HttpMethods.GET)
        .path(request.assetPath())
        .build();
    Context nested = new Context(getRepository(), nestedRequest);
    nested.getAttributes().set(TerraformRequest.class, request);
    nested.getAttributes().set(AssetKind.class, request.assetKind());
    return nested;
  }

  private static String hostOf(final URI uri) {
    return uri.getHost() == null ? "" : uri.getHost().toLowerCase(Locale.ROOT);
  }

  private static URI withSlash(final URI uri) {
    String value = uri.toString();
    return value.endsWith("/") ? uri : URI.create(value + "/");
  }

  private static String resolveUrl(final URI base, final String url) {
    return base.resolve(url).toString();
  }

  static String baseName(final String url) {
    String path = URI.create(url.strip()).getPath();
    if (path == null) {
      return "";
    }
    return path.substring(path.lastIndexOf('/') + 1);
  }

  private static String relativeModulePath(final ModuleCoordinates module) {
    return module.namespace() + "/" + module.name() + "/" + module.system() + "/";
  }

  private static String relativeProviderPath(final ProviderCoordinates provider) {
    return provider.namespace() + "/" + provider.type() + "/";
  }

  private TerraformContentFacet content() {
    return facet(TerraformContentFacet.class);
  }
}
