package org.globalart.astrum.repository.huggingface.internal.proxy;

import java.io.IOException;
import java.io.InputStream;
import java.net.URI;
import java.util.Map;
import java.util.Objects;
import java.util.Optional;
import java.util.Set;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.ExecutionException;
import java.util.stream.Collectors;

import javax.annotation.Nonnull;
import javax.annotation.Nullable;

import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.InvalidContentException;
import org.globalart.astrum.repository.cache.CacheController;
import org.globalart.astrum.repository.cache.CacheInfo;
import org.globalart.astrum.repository.cache.NegativeCacheFacet;
import org.globalart.astrum.repository.cache.NegativeCacheKey;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.facet.ContentProxyFacetSupport;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.http.HttpMethods;
import org.globalart.astrum.repository.httpclient.HttpClientFacet;
import org.globalart.astrum.repository.httpclient.RemoteBlockedIOException;
import org.globalart.astrum.repository.huggingface.HuggingFaceFormat;
import org.globalart.astrum.repository.huggingface.internal.HuggingFaceContentFacet;
import org.globalart.astrum.repository.huggingface.internal.HuggingFaceException;
import org.globalart.astrum.repository.huggingface.internal.HuggingFaceFile;
import org.globalart.astrum.repository.huggingface.internal.HuggingFacePaths;
import org.globalart.astrum.repository.huggingface.internal.RepoType;
import org.globalart.astrum.repository.huggingface.internal.RevisionInfo;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.repository.view.Status;
import org.globalart.astrum.validation.ssrf.AntiSsrfService;

import com.google.common.cache.Cache;
import com.google.common.cache.CacheBuilder;
import org.apache.http.Header;
import org.apache.http.HttpHeaders;
import org.apache.http.HttpResponse;
import org.apache.http.client.HttpClient;
import org.apache.http.client.config.RequestConfig;
import org.apache.http.client.methods.Configurable;
import org.apache.http.client.methods.HttpGet;
import org.apache.http.client.methods.HttpHead;
import org.apache.http.client.methods.HttpRequestBase;
import org.apache.http.client.utils.HttpClientUtils;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Proxies the Hub. The information of a revision is fetched with the files it lists ({@code blobs=true}); for a
 * branch or tag it is metadata, refreshed after the metadata max age, and its copy under the commit it resolved to
 * is kept like content. Files are fetched and stored by commit only, so a commit is served from the cache once its
 * files were fetched. Redirects are followed here: the token of the repository goes to the Hub only, never to the
 * storage the Hub redirects LFS files to.
 */
@Facet.Exposed
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
@Qualifier(HuggingFaceFormat.NAME)
public class HuggingFaceProxyFacet
    extends ContentProxyFacetSupport
{
  private static final CacheController IMMUTABLE = new CacheController(-1, null);

  private static final int MAX_REDIRECTS = 5;

  private static final int MAX_REVISION_BYTES = 64 * 1024 * 1024;

  private final Cache<String, RevisionInfo> revisions = CacheBuilder.newBuilder().maximumSize(32).build();

  private final Cache<String, Map<String, String>> resolutions = CacheBuilder.newBuilder().maximumSize(4096).build();

  private AntiSsrfService redirectValidator;

  @Autowired
  public void setRedirectValidator(final AntiSsrfService redirectValidator) {
    this.redirectValidator = checkNotNull(redirectValidator);
  }

  /**
   * The information of a revision, from the cache while it is fresh.
   *
   * @throws HuggingFaceException when the remote does not know the repository or the revision
   */
  public RevisionInfo revision(
      final Context context,
      final RepoType type,
      final String repoId,
      final String revision) throws IOException
  {
    ProxyTarget target = ProxyTarget.revision(type, repoId, revision);
    Context proxyContext = proxyContext(context, target);
    NegativeCacheFacet negativeCache = negativeCache();
    NegativeCacheKey key = negativeCache != null ? negativeCache.getCacheKey(proxyContext) : null;
    Status missing = key != null ? negativeCache.get(key) : null;
    if (missing != null && getCachedContent(proxyContext) == null) {
      throw HuggingFaceException.decode(missing.getCode(), missing.getMessage());
    }
    Content content = get(proxyContext);
    if (content == null) {
      HuggingFaceException failure = failure(proxyContext, target);
      if (key != null && failure.getStatus() == 404) {
        negativeCache.put(key, Status.failure(404, failure.encode()));
      }
      throw failure;
    }
    if (missing != null) {
      negativeCache.invalidate(key);
    }
    RevisionInfo info = parse(content);
    if (!HuggingFacePaths.isCommit(revision)) {
      info.sha().filter(HuggingFacePaths::isCommit).ifPresent(sha -> remember(type, repoId, revision, sha));
    }
    return info;
  }

  /**
   * A file at a commit, fetched and stored when it is not cached yet.
   */
  public Content file(final Context context, final HuggingFaceFile file) throws IOException {
    ProxyTarget target = ProxyTarget.file(file);
    Context proxyContext = proxyContext(context, target);
    if (getCachedContent(proxyContext) == null) {
      Optional<FluentAsset> reused = content()
          .reuseFile(file.withRemote(revisionsOf(file.type(), file.repoId(), file.commit()), null, null, -1));
      if (reused.isPresent()) {
        log.debug("Reusing the stored content of {} for commit {}", file.path(), file.commit());
        Content stored = reused.get().download();
        stored.getAttributes().set(Content.CONTENT_ETAG, file.etag());
        stored.getAttributes().set(CacheInfo.class, IMMUTABLE.current());
        reused.get().markAsCached(stored);
        return reused.get().download();
      }
    }
    Content content = get(proxyContext);
    if (content == null) {
      throw failure(proxyContext, target);
    }
    return content;
  }

  public Optional<Content> cachedFile(final HuggingFaceFile file) {
    return content().getAsset(file.assetPath()).map(FluentAsset::download);
  }

  public Optional<Content> cachedAsset(final String path) {
    return content().getAsset(path).map(FluentAsset::download);
  }

  /**
   * The branches and tags this repository saw resolve to the commit.
   */
  public Set<String> revisionsOf(final RepoType type, final String repoId, final String commit) {
    Map<String, String> known = resolutions.getIfPresent(type.plural() + "/" + repoId);
    if (known == null) {
      return Set.of();
    }
    return known.entrySet().stream()
        .filter(entry -> commit.equals(entry.getValue()))
        .map(Map.Entry::getKey)
        .collect(Collectors.toSet());
  }

  /**
   * Asks the Hub for the headers of a file without fetching it. A redirect within the Hub (a renamed repository) is
   * followed, a redirect to its storage is the answer.
   */
  public HubResponse head(final RepoType type, final String repoId, final String commit, final String path)
      throws IOException
  {
    HttpClient client = facet(HttpClientFacet.class).getHttpClient();
    URI uri = getRemoteUrl().resolve(HuggingFacePaths.fileUrl(type, repoId, commit, path));
    for (int hop = 0; hop <= MAX_REDIRECTS; hop++) {
      validateRedirect(uri);
      HttpHead head = new HttpHead(uri);
      prepare(client, head, true);
      HttpResponse response = client.execute(head);
      try {
        HubResponse hub = HubResponse.of(response);
        Header location = response.getFirstHeader(HttpHeaders.LOCATION);
        if (!hub.isRedirect() || location == null) {
          return hub;
        }
        URI next = uri.resolve(location.getValue());
        if (!isRemote(next)) {
          return hub.found();
        }
        uri = next;
      }
      finally {
        HttpClientUtils.closeQuietly(response);
      }
    }
    throw new IOException("Too many redirects from " + uri);
  }

  @Nullable
  @Override
  protected Content getCachedContent(final Context context) {
    return content().getAsset(target(context).assetPath()).map(FluentAsset::download).orElse(null);
  }

  @Nonnull
  @Override
  protected CacheController getCacheController(@Nonnull final Context context) {
    return target(context).immutable() ? IMMUTABLE : cacheControllerHolder.getMetadataCacheController();
  }

  @Override
  protected String getUrl(@Nonnull final Context context) {
    return getRemoteUrl().resolve(target(context).remotePath()).toString();
  }

  @Override
  protected HttpRequestBase buildFetchHttpRequest(final URI uri, final Context context) {
    HttpRequestBase request = super.buildFetchHttpRequest(uri, context);
    prepare(facet(HttpClientFacet.class).getHttpClient(), request, isRemote(uri));
    return request;
  }

  @Override
  protected HttpResponse execute(
      final Context context,
      final HttpClient client,
      final HttpRequestBase request) throws IOException
  {
    URI uri = request.getURI();
    HttpResponse response = super.execute(context, client, request);
    if (isRemote(uri)) {
      context.getAttributes().set(HubResponse.class, HubResponse.of(response));
    }
    for (int hop = 0; hop < MAX_REDIRECTS && isRedirect(response); hop++) {
      Header location = response.getFirstHeader(HttpHeaders.LOCATION);
      if (location == null) {
        break;
      }
      URI next = uri.resolve(location.getValue());
      HttpClientUtils.closeQuietly(response);
      validateRedirect(next);
      HttpRequestBase follow = HttpMethods.HEAD.equals(request.getMethod()) ? new HttpHead(next) : new HttpGet(next);
      prepare(client, follow, isRemote(next));
      response = super.execute(context, client, follow);
      uri = next;
      if (isRemote(next)) {
        context.getAttributes().set(HubResponse.class, HubResponse.of(response));
      }
    }
    return response;
  }

  @Override
  protected Content store(final Context context, final Content content) throws IOException {
    ProxyTarget target = target(context);
    return target.file() != null ? storeFile(context, target.file(), content) : storeRevision(target, content);
  }

  private Content storeRevision(final ProxyTarget target, final Content content) throws IOException {
    byte[] json = read(content);
    RevisionInfo info;
    try {
      info = RevisionInfo.parse(json);
    }
    catch (IllegalArgumentException e) {
      throw new InvalidContentException("The remote answered no revision information for " + target.describe(), e);
    }
    FluentAsset asset = content().saveRevision(target.assetPath(), json);
    asset.markAsCached(content);
    if (!target.immutable()) {
      info.sha().filter(HuggingFacePaths::isCommit).ifPresent(sha -> {
        String commitPath = HuggingFacePaths.revisionPath(target.type(), target.repoId(), sha);
        if (content().getAsset(commitPath).isEmpty()) {
          content().saveRevision(commitPath, json).markAsCached(IMMUTABLE.current());
        }
      });
    }
    Content stored = asset.download();
    String key = parsedKey(stored);
    if (key != null) {
      revisions.put(key, info);
    }
    return stored;
  }

  private Content storeFile(final Context context, final HuggingFaceFile file, final Content content) {
    HubResponse hub = context.getAttributes().get(HubResponse.class);
    String remoteEtag = hub != null ? hub.etag() : null;
    String remoteSha256 = hub != null && hub.linked() && HuggingFacePaths.isSha256(hub.etag()) ? hub.etag() : null;
    long remoteSize = hub != null && hub.linked() ? hub.size() : -1;
    HuggingFaceFile resolved = file.withRemote(revisionsOf(file.type(), file.repoId(), file.commit()), remoteEtag,
        remoteSha256, remoteSize);
    FluentAsset asset = content().saveFile(resolved, content);
    if (resolved.etag() != null) {
      content.getAttributes().set(Content.CONTENT_ETAG, resolved.etag());
    }
    asset.markAsCached(content);
    return asset.download();
  }

  private RevisionInfo parse(final Content content) throws IOException {
    String key = parsedKey(content);
    if (key == null) {
      return parseRevision(read(content));
    }
    try {
      return revisions.get(key, () -> parseRevision(read(content)));
    }
    catch (ExecutionException e) {
      if (e.getCause() instanceof IOException io) {
        throw io;
      }
      throw new IOException(e.getCause());
    }
  }

  private static RevisionInfo parseRevision(final byte[] json) {
    try {
      return RevisionInfo.parse(json);
    }
    catch (IllegalArgumentException e) {
      throw new InvalidContentException("Cached revision information is not valid", e);
    }
  }

  @Nullable
  private static String parsedKey(final Content content) {
    Asset asset = content.getAttributes().get(Asset.class);
    if (asset == null) {
      return null;
    }
    return asset.path() + "@" + asset.blob().map(blob -> blob.blobRef().toString()).orElse("");
  }

  private static byte[] read(final Content content) throws IOException {
    try (InputStream in = content.openInputStream()) {
      byte[] bytes = in.readNBytes(MAX_REVISION_BYTES + 1);
      if (bytes.length > MAX_REVISION_BYTES) {
        throw new InvalidContentException("Revision information is larger than " + MAX_REVISION_BYTES + " bytes");
      }
      return bytes;
    }
  }

  private void remember(final RepoType type, final String repoId, final String revision, final String sha) {
    try {
      resolutions.get(type.plural() + "/" + repoId, ConcurrentHashMap::new).put(revision, sha);
    }
    catch (ExecutionException e) {
      log.debug("Unable to remember that {} of {} is {}", revision, repoId, sha, e);
    }
  }

  private HuggingFaceException failure(final Context proxyContext, final ProxyTarget target) {
    HubResponse hub = proxyContext.getAttributes().get(HubResponse.class);
    if (hub != null && hub.status() >= 400) {
      String message = hub.errorMessage() != null ? hub.errorMessage()
          : "The remote answered " + hub.status() + " for " + target.describe();
      return new HuggingFaceException(hub.status(), hub.errorCode(), message);
    }
    if (hub != null && hub.isRedirect()) {
      return new HuggingFaceException(502, null, "The remote redirected too often for " + target.describe());
    }
    return new HuggingFaceException(404, null, target.describe() + " is not available on the remote");
  }

  private Context proxyContext(final Context context, final ProxyTarget target) {
    Request request = new Request.Builder()
        .action(HttpMethods.GET)
        .path(target.assetPath())
        .build();
    Context proxyContext = new Context(context.getRepository(), request);
    proxyContext.getAttributes().backing().putAll(context.getAttributes().backing());
    proxyContext.getAttributes().remove(HubResponse.class);
    proxyContext.getAttributes().set(ProxyTarget.class, target);
    return proxyContext;
  }

  private static ProxyTarget target(final Context context) {
    return context.getAttributes().require(ProxyTarget.class);
  }

  private void prepare(final HttpClient client, final HttpRequestBase request, final boolean remote) {
    if (remote) {
      String token = facet(HttpClientFacet.class).getBearerToken();
      if (token != null && !token.isBlank()) {
        request.setHeader(HttpHeaders.AUTHORIZATION, "Bearer " + token);
      }
    }
    else {
      request.removeHeaders(HttpHeaders.AUTHORIZATION);
    }
    RequestConfig base = request.getConfig();
    if (base == null && client instanceof Configurable configurable) {
      base = configurable.getConfig();
    }
    request.setConfig(RequestConfig.copy(base != null ? base : RequestConfig.DEFAULT)
        .setRedirectsEnabled(false)
        .build());
  }

  private boolean isRemote(final URI uri) {
    URI remote = getRemoteUrl();
    return Objects.equals(lower(remote.getScheme()), lower(uri.getScheme()))
        && Objects.equals(lower(remote.getHost()), lower(uri.getHost()))
        && port(remote) == port(uri);
  }

  private static int port(final URI uri) {
    if (uri.getPort() >= 0) {
      return uri.getPort();
    }
    return "https".equalsIgnoreCase(uri.getScheme()) ? 443 : 80;
  }

  @Nullable
  private static String lower(@Nullable final String value) {
    return value == null ? null : value.toLowerCase(java.util.Locale.ROOT);
  }

  private static boolean isRedirect(final HttpResponse response) {
    int status = response.getStatusLine().getStatusCode();
    return status == 301 || status == 302 || status == 303 || status == 307 || status == 308;
  }

  private void validateRedirect(final URI uri) throws RemoteBlockedIOException {
    if (redirectValidator == null || uri.getHost() == null) {
      return;
    }
    try {
      redirectValidator.validateHost(uri.getHost());
    }
    catch (RuntimeException e) {
      throw new RemoteBlockedIOException("Access to remote " + uri + " blocked: " + e.getMessage());
    }
  }

  @Nullable
  private NegativeCacheFacet negativeCache() {
    return optionalFacet(NegativeCacheFacet.class).orElse(null);
  }

  private HuggingFaceContentFacet content() {
    return facet(HuggingFaceContentFacet.class);
  }
}
