package org.globalart.astrum.repository.huggingface.internal.proxy;

import java.io.IOException;
import java.io.InputStream;
import java.net.URLDecoder;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.Set;
import java.util.TreeSet;

import javax.annotation.Nonnull;
import javax.annotation.Nullable;

import org.globalart.astrum.repository.InvalidContentException;
import org.globalart.astrum.repository.http.HttpMethods;
import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.httpclient.RemoteBlockedIOException;
import org.globalart.astrum.repository.huggingface.internal.HeadPayload;
import org.globalart.astrum.repository.huggingface.internal.HuggingFaceException;
import org.globalart.astrum.repository.huggingface.internal.HuggingFaceFile;
import org.globalart.astrum.repository.huggingface.internal.HuggingFacePaths;
import org.globalart.astrum.repository.huggingface.internal.HuggingFaceRequest;
import org.globalart.astrum.repository.huggingface.internal.HuggingFaceResponses;
import org.globalart.astrum.repository.huggingface.internal.RepoFile;
import org.globalart.astrum.repository.huggingface.internal.RevisionInfo;
import org.globalart.astrum.repository.proxy.ProxyServiceException;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.Response;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.apache.http.HttpResponse;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.huggingface.internal.HuggingFaceException.ENTRY_NOT_FOUND;
import static org.globalart.astrum.repository.huggingface.internal.HuggingFaceResponses.X_ERROR_CODE;
import static org.globalart.astrum.repository.huggingface.internal.HuggingFaceResponses.X_ERROR_MESSAGE;

/**
 * Answers the requests of Hub clients: revision information, tree listings and path information from the cached
 * revision, and files by commit with the headers ({@code X-Repo-Commit}, {@code ETag}, {@code X-Linked-Etag},
 * {@code X-Linked-Size}) clients key their cache by. A branch or tag is resolved to its commit first.
 */
@Component
public class HuggingFaceProxyHandler
    implements Handler
{
  private static final Logger log = LoggerFactory.getLogger(HuggingFaceProxyHandler.class);

  private static final ObjectMapper MAPPER = new ObjectMapper();

  private static final int MAX_FORM_BYTES = 1024 * 1024;

  @Nonnull
  @Override
  public Response handle(@Nonnull final Context context) throws Exception {
    HuggingFaceRequest request = context.getAttributes().require(HuggingFaceRequest.class);
    HuggingFaceProxyFacet proxy = context.getRepository().facet(HuggingFaceProxyFacet.class);
    try {
      return switch (request.kind()) {
        case INFO -> HuggingFaceResponses.json(revision(context, proxy, request).raw());
        case TREE -> tree(context, proxy, request);
        case PATHS_INFO -> pathsInfo(context, proxy, request);
        case FILE -> file(context, proxy, request);
        case STORED -> stored(proxy, request);
      };
    }
    catch (HuggingFaceException e) {
      return HuggingFaceResponses.error(e);
    }
    catch (ProxyServiceException e) {
      return remoteFailure(e.getHttpResponse());
    }
    catch (RemoteBlockedIOException e) {
      return HttpResponses.notFound(e.getMessage());
    }
    catch (InvalidContentException e) {
      log.warn("Invalid content from the remote of {} for {}: {}", context.getRepository().getName(),
          context.getRequest().getPath(), e.getMessage());
      return HttpResponses.badGateway(e.getMessage());
    }
    catch (IOException e) {
      log.debug("Remote of {} failed for {}", context.getRepository().getName(), context.getRequest().getPath(), e);
      return HttpResponses.badGateway();
    }
  }

  private static RevisionInfo revision(
      final Context context,
      final HuggingFaceProxyFacet proxy,
      final HuggingFaceRequest request) throws IOException
  {
    String revision = request.revision() != null ? request.revision() : HuggingFacePaths.DEFAULT_REVISION;
    return proxy.revision(context, request.type(), request.repoId(), revision);
  }

  private static Response tree(
      final Context context,
      final HuggingFaceProxyFacet proxy,
      final HuggingFaceRequest request) throws IOException
  {
    RevisionInfo info = revision(context, proxy, request);
    boolean recursive = isTrue(context.getRequest().getParameters().get("recursive"));
    return info.tree(request.path(), recursive)
        .map(HuggingFaceResponses::json)
        .orElseThrow(() -> new HuggingFaceException(404, ENTRY_NOT_FOUND,
            request.path() + " is not a directory of " + request.repoId() + " at " + request.revision()));
  }

  private static Response pathsInfo(
      final Context context,
      final HuggingFaceProxyFacet proxy,
      final HuggingFaceRequest request) throws IOException
  {
    List<String> paths = requestedPaths(context);
    RevisionInfo info = revision(context, proxy, request);
    return HuggingFaceResponses.json(info.pathsInfo(paths));
  }

  private static Response file(
      final Context context,
      final HuggingFaceProxyFacet proxy,
      final HuggingFaceRequest request) throws IOException
  {
    String revision = request.revision();
    Set<String> revisions = new TreeSet<>();
    String commit = revision;
    if (!HuggingFacePaths.isCommit(revision)) {
      commit = revision(context, proxy, request).sha()
          .filter(HuggingFacePaths::isCommit)
          .orElseThrow(() -> new HuggingFaceException(502, null,
              "The remote answered no commit for " + request.repoId() + " at " + revision));
      revisions.add(revision);
    }
    RevisionInfo atCommit = proxy.revision(context, request.type(), request.repoId(), commit);
    Optional<RepoFile> listed = atCommit.file(request.path());
    if (listed.isEmpty() && atCommit.listsFiles()) {
      throw HuggingFaceException.entryNotFound(commit, request.path());
    }
    revisions.addAll(proxy.revisionsOf(request.type(), request.repoId(), commit));
    HuggingFaceFile file = HuggingFaceFile.of(request.type(), request.repoId(), commit, request.path(), revisions,
        listed.orElse(null));

    if (HttpMethods.HEAD.equals(context.getRequest().getAction())) {
      return head(proxy, file);
    }
    Content content = proxy.file(context, file);
    return HuggingFaceResponses.file(content, commit, etag(content, file), content.getSize());
  }

  private static Response head(final HuggingFaceProxyFacet proxy, final HuggingFaceFile file) throws IOException {
    Optional<Content> cached = proxy.cachedFile(file);
    if (cached.isPresent()) {
      Content content = cached.get();
      return HuggingFaceResponses.file(content, file.commit(), etag(content, file), content.getSize());
    }
    if (file.etag() != null && file.size() >= 0) {
      return HuggingFaceResponses.file(new HeadPayload(file.size(), null), file.commit(), file.etag(), file.size());
    }
    HubResponse hub = proxy.head(file.type(), file.repoId(), file.commit(), file.path());
    if (hub.status() >= 400) {
      throw new HuggingFaceException(hub.status(), hub.errorCode(),
          hub.errorMessage() != null ? hub.errorMessage() : file.path() + " is not available on the remote",
          hub.commit());
    }
    if (hub.status() != 200) {
      throw new HuggingFaceException(502, null, "The remote answered " + hub.status() + " for " + file.path());
    }
    return HuggingFaceResponses.file(new HeadPayload(hub.size(), null), file.commit(), hub.etag(), hub.size());
  }

  private static Response stored(final HuggingFaceProxyFacet proxy, final HuggingFaceRequest request) {
    Optional<Content> content = proxy.cachedAsset(request.path());
    if (content.isEmpty()) {
      return HttpResponses.notFound();
    }
    return HttpResponses.ok(content.get());
  }

  @Nullable
  private static String etag(final Content content, final HuggingFaceFile file) {
    String stored = content.getAttributes().get(Content.CONTENT_ETAG, String.class);
    return file.etag() != null ? file.etag() : stored;
  }

  private static Response remoteFailure(final HttpResponse response) {
    int status = response.getStatusLine().getStatusCode();
    if (status == 401 || status == 403) {
      String code = header(response, X_ERROR_CODE);
      String message = header(response, X_ERROR_MESSAGE);
      return HuggingFaceResponses.error(new HuggingFaceException(status, code, message != null ? message
          : "The remote refused the request; set a token in the HTTP authentication of the repository"));
    }
    return HttpResponses.badGateway("The remote answered " + status);
  }

  @Nullable
  private static String header(final HttpResponse response, final String name) {
    return response.getFirstHeader(name) != null ? response.getFirstHeader(name).getValue() : null;
  }

  private static boolean isTrue(@Nullable final String value) {
    return value != null && ("true".equalsIgnoreCase(value) || "1".equals(value));
  }

  static List<String> requestedPaths(final Context context) throws IOException {
    List<String> paths = new ArrayList<>(context.getRequest().getParameters().getAll("paths"));
    Payload payload = context.getRequest().getPayload();
    if (payload == null) {
      return paths;
    }
    byte[] body;
    try (InputStream in = payload.openInputStream()) {
      body = in.readNBytes(MAX_FORM_BYTES);
    }
    String contentType = payload.getContentType() != null ? payload.getContentType() : "";
    if (contentType.startsWith("application/json")) {
      JsonNode node = MAPPER.readTree(body);
      JsonNode values = node != null ? node.get("paths") : null;
      if (values != null && values.isArray()) {
        values.forEach(value -> paths.add(value.asText()));
      }
      return paths;
    }
    for (String pair : new String(body, StandardCharsets.UTF_8).split("&")) {
      int equals = pair.indexOf('=');
      if (equals > 0 && "paths".equals(URLDecoder.decode(pair.substring(0, equals), StandardCharsets.UTF_8))) {
        paths.add(URLDecoder.decode(pair.substring(equals + 1), StandardCharsets.UTF_8));
      }
    }
    return paths;
  }
}
