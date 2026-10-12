package org.globalart.astrum.repository.docker.internal.v2;

import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Optional;

import javax.annotation.Nonnull;
import javax.annotation.Nullable;

import org.globalart.astrum.repository.config.WritePolicy;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.docker.DockerContentFacet;
import org.globalart.astrum.repository.docker.internal.v2.DockerContentResponses.TooLargeException;
import org.globalart.astrum.repository.docker.internal.v2.DockerManifest.InvalidManifestException;
import org.globalart.astrum.repository.http.HttpMethods;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.payloads.TempBlob;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

@Component
public class DockerHostedHandler
    implements Handler
{
  private static final Logger log = LoggerFactory.getLogger(DockerHostedHandler.class);

  private static final String TAG_PARAMETER = "tag";

  private final DockerUploadStore uploadStore;

  private final DockerBlobMounter blobMounter;

  private final long maxManifestSize;

  @Autowired
  public DockerHostedHandler(
      final DockerUploadStore uploadStore,
      final DockerBlobMounter blobMounter,
      @Value("${astrum.docker.manifest.maxSize:4194304}") final long maxManifestSize)
  {
    this.uploadStore = checkNotNull(uploadStore);
    this.blobMounter = checkNotNull(blobMounter);
    this.maxManifestSize = maxManifestSize;
  }

  @Nonnull
  @Override
  public Response handle(@Nonnull final Context context) throws Exception {
    DockerRoute route = DockerRoute.get(context);
    String action = context.getRequest().getAction();
    switch (route.getType()) {
      case MANIFEST:
        return manifest(context, route, action);
      case BLOB:
        return blob(context, route, action);
      case UPLOADS:
        return startUpload(context, route);
      case UPLOAD:
        return upload(context, route, action);
      default:
        return DockerResponses.error(DockerError.UNSUPPORTED);
    }
  }

  private static DockerContentFacet content(final Context context) {
    return context.getRepository().facet(DockerContentFacet.class);
  }

  private Response manifest(final Context context, final DockerRoute route, final String action) throws IOException {
    switch (action) {
      case HttpMethods.GET:
      case HttpMethods.HEAD:
        return content(context).findManifest(route.getName(), route.getReference())
            .map(asset -> DockerContentResponses.manifest(asset, route.getReference()))
            .orElseGet(() -> manifestUnknown(context, route));
      case HttpMethods.PUT:
        return putManifest(context, route);
      case HttpMethods.DELETE:
        return deleteManifest(context, route);
      default:
        return DockerResponses.error(DockerError.UNSUPPORTED);
    }
  }

  private static Response manifestUnknown(final Context context, final DockerRoute route) {
    return DockerResponses.error(DockerError.MANIFEST_UNKNOWN, DockerError.MANIFEST_UNKNOWN.message(),
        Map.of("name", DockerLocations.externalName(context, route.getName()), "reference", route.getReference()));
  }

  private Response putManifest(final Context context, final DockerRoute route) throws IOException {
    DockerContentFacet content = content(context);
    WritePolicy policy = content.configuredWritePolicy();
    if (!policy.checkCreateAllowed()) {
      return readOnly(context);
    }
    Request request = context.getRequest();
    if (request.getPayload() == null) {
      return DockerResponses.error(DockerError.MANIFEST_INVALID, "manifest body is required");
    }
    byte[] bytes;
    try {
      bytes = DockerContentResponses.read(request.getPayload(), maxManifestSize);
    }
    catch (TooLargeException e) {
      return DockerResponses.error(413, DockerError.MANIFEST_INVALID,
          "manifest exceeds the maximum size of " + maxManifestSize + " bytes", null);
    }
    String name = route.getName();
    String reference = route.getReference();
    if (route.isDigestReference() && !DockerDigest.isSupported(reference)) {
      return DockerResponses.error(DockerError.DIGEST_INVALID, "unsupported digest " + reference);
    }
    String digest = route.isDigestReference() ? DockerDigest.of(bytes, reference) : DockerDigest.sha256(bytes);
    if (route.isDigestReference() && !reference.equals(digest)) {
      return DockerResponses.error(DockerError.DIGEST_INVALID,
          "manifest digest " + digest + " does not match " + reference);
    }

    DockerManifest manifest;
    try {
      manifest = DockerManifest.parse(bytes, request.getHeaders().get("Content-Type"));
    }
    catch (InvalidManifestException e) {
      return DockerResponses.error(DockerError.MANIFEST_INVALID, e.getMessage());
    }
    if (manifest.isSchema1()) {
      return DockerResponses.error(DockerError.MANIFEST_INVALID,
          "schema1 manifests are not supported, push an image manifest or an index");
    }
    for (String blob : manifest.getBlobs()) {
      if (content.findBlob(blob).isEmpty()) {
        return DockerResponses.error(DockerError.MANIFEST_BLOB_UNKNOWN,
            "blob " + blob + " referenced by the manifest is unknown", Map.of("digest", blob));
      }
    }
    for (String child : manifest.getManifests()) {
      if (content.findManifest(name, child).isEmpty()) {
        return DockerResponses.error(400, DockerError.MANIFEST_UNKNOWN,
            "manifest " + child + " referenced by the index is unknown", Map.of("digest", child));
      }
    }

    List<String> tags = new ArrayList<>();
    if (!route.isDigestReference()) {
      tags.add(reference);
    }
    for (String tag : request.getParameters().getAll(TAG_PARAMETER)) {
      if (!DockerRouteMatcher.isValidTag(tag)) {
        return DockerResponses.error(DockerError.MANIFEST_INVALID, "invalid tag " + tag, Map.of("tag", tag));
      }
      if (!tags.contains(tag)) {
        tags.add(tag);
      }
    }
    for (String tag : tags) {
      Optional<FluentAsset> existing = content.findManifest(name, tag);
      if (existing.isPresent() && !digest.equals(DockerContentFacet.digestOf(existing.get()))
          && !tagUpdateAllowed(content, policy, tag)) {
        return DockerResponses.error(400, DockerError.DENIED,
            "repository " + context.getRepository().getName() + " does not allow updating tag " + tag
                + " of " + DockerLocations.externalName(context, name), null);
      }
    }

    content.saveManifest(name, digest, bytes, manifest);
    for (String tag : tags) {
      content.saveTag(name, tag, digest, bytes, manifest);
    }

    Response.Builder response = DockerResponses.success(201)
        .header("Location", DockerLocations.manifest(context, name, digest))
        .header(DockerResponses.CONTENT_DIGEST_HEADER, digest)
        .header("Content-Length", "0");
    if (manifest.getSubject() != null) {
      response.header(DockerResponses.OCI_SUBJECT_HEADER, manifest.getSubject());
    }
    if (route.isDigestReference() && !tags.isEmpty()) {
      response.header(DockerResponses.OCI_TAG_HEADER, String.join(", ", tags));
    }
    return response.build();
  }

  private static boolean tagUpdateAllowed(
      final DockerContentFacet content,
      final WritePolicy policy,
      final String tag)
  {
    if (policy.checkUpdateAllowed()) {
      return true;
    }
    return policy == WritePolicy.ALLOW_ONCE && DockerContentFacet.LATEST.equals(tag) && content.latestPolicy();
  }

  private Response deleteManifest(final Context context, final DockerRoute route) {
    DockerContentFacet content = content(context);
    if (!content.configuredWritePolicy().checkDeleteAllowed()) {
      return readOnly(context);
    }
    boolean deleted = route.isDigestReference()
        ? content.deleteManifest(route.getName(), route.getReference())
        : content.deleteTag(route.getName(), route.getReference());
    return deleted ? DockerResponses.empty(202) : manifestUnknown(context, route);
  }

  private Response blob(final Context context, final DockerRoute route, final String action) {
    DockerContentFacet content = content(context);
    String digest = route.getReference();
    switch (action) {
      case HttpMethods.GET:
      case HttpMethods.HEAD:
        return content.findBlob(digest)
            .map(asset -> DockerContentResponses.blob(asset.download(), digest))
            .orElseGet(() -> blobUnknown(digest));
      case HttpMethods.DELETE:
        if (!content.configuredWritePolicy().checkDeleteAllowed()) {
          return readOnly(context);
        }
        return content.deleteBlob(digest) ? DockerResponses.empty(202) : blobUnknown(digest);
      default:
        return DockerResponses.error(DockerError.UNSUPPORTED);
    }
  }

  private static Response blobUnknown(final String digest) {
    return DockerResponses.error(DockerError.BLOB_UNKNOWN, DockerError.BLOB_UNKNOWN.message(),
        Map.of("digest", digest));
  }

  private static Response readOnly(final Context context) {
    return DockerResponses.error(DockerError.DENIED,
        "repository " + context.getRepository().getName() + " is read-only");
  }

  private Response startUpload(final Context context, final DockerRoute route) throws IOException {
    if (!HttpMethods.POST.equals(context.getRequest().getAction())) {
      return DockerResponses.error(DockerError.UNSUPPORTED);
    }
    DockerContentFacet content = content(context);
    if (!content.configuredWritePolicy().checkCreateAllowed()) {
      return readOnly(context);
    }
    String name = route.getName();
    String mount = context.getRequest().getParameters().get("mount");
    if (mount != null && DockerDigest.isDigest(mount)) {
      if (content.findBlob(mount).isPresent()
          || blobMounter.mount(context, mount, context.getRequest().getParameters().get("from"))) {
        return blobCreated(context, name, mount);
      }
    }
    String digest = context.getRequest().getParameters().get("digest");
    if (digest != null) {
      return monolithicUpload(context, name, digest);
    }
    String uuid = uploadStore.start(context.getRepository());
    return uploadStatus(context, name, uuid, 0L, 202);
  }

  private Response monolithicUpload(final Context context, final String name, final String digest) {
    if (!DockerDigest.isSupported(digest)) {
      return DockerResponses.error(DockerError.DIGEST_INVALID, "unsupported digest " + digest);
    }
    Payload payload = context.getRequest().getPayload();
    if (payload == null) {
      return DockerResponses.error(DockerError.BLOB_UPLOAD_INVALID, "blob body is required");
    }
    DockerContentFacet content = content(context);
    try (TempBlob blob = content.ingest(payload, digest)) {
      return finishBlob(context, name, digest, blob);
    }
  }

  private Response upload(final Context context, final DockerRoute route, final String action) throws IOException {
    String name = route.getName();
    String uuid = route.getReference();
    Optional<Path> file = uploadStore.find(context.getRepository(), uuid);
    if (file.isEmpty()) {
      return DockerResponses.error(DockerError.BLOB_UPLOAD_UNKNOWN, DockerError.BLOB_UPLOAD_UNKNOWN.message(),
          Map.of("uuid", uuid));
    }
    Path path = file.get();
    switch (action) {
      case HttpMethods.GET:
        return uploadStatus(context, name, uuid, uploadStore.size(path), 204);
      case HttpMethods.DELETE:
        uploadStore.delete(path);
        return DockerResponses.empty(204);
      case HttpMethods.PATCH:
        return patchUpload(context, name, uuid, path);
      case HttpMethods.PUT:
        return completeUpload(context, name, uuid, path);
      default:
        return DockerResponses.error(DockerError.UNSUPPORTED);
    }
  }

  private Response patchUpload(
      final Context context,
      final String name,
      final String uuid,
      final Path path) throws IOException
  {
    long current = uploadStore.size(path);
    long[] range = parseContentRange(context.getRequest().getHeaders().get("Content-Range"));
    if (range != null && range[0] != current) {
      return outOfOrder(context, name, uuid, current, range[0]);
    }
    long size = appendBody(context, path);
    if (range != null && range[1] >= range[0] && size != range[1] + 1) {
      return DockerResponses.error(DockerError.SIZE_INVALID,
          "chunk ended at " + (size - 1) + " but Content-Range declared " + range[1]);
    }
    return uploadStatus(context, name, uuid, size, 202);
  }

  private Response completeUpload(
      final Context context,
      final String name,
      final String uuid,
      final Path path) throws IOException
  {
    String digest = context.getRequest().getParameters().get("digest");
    if (digest == null) {
      return DockerResponses.error(DockerError.DIGEST_INVALID, "digest parameter is required");
    }
    if (!DockerDigest.isSupported(digest)) {
      return DockerResponses.error(DockerError.DIGEST_INVALID, "unsupported digest " + digest);
    }
    long current = uploadStore.size(path);
    long[] range = parseContentRange(context.getRequest().getHeaders().get("Content-Range"));
    if (range != null && range[0] != current) {
      return outOfOrder(context, name, uuid, current, range[0]);
    }
    appendBody(context, path);
    DockerContentFacet content = content(context);
    try (TempBlob blob = content.ingest(path, digest)) {
      return finishBlob(context, name, digest, blob);
    }
    finally {
      uploadStore.delete(path);
    }
  }

  private static Response outOfOrder(
      final Context context,
      final String name,
      final String uuid,
      final long current,
      final long start)
  {
    Response error = DockerResponses.error(DockerError.RANGE_INVALID,
        "upload is at offset " + current + ", chunk starts at " + start);
    error.getHeaders().replace("Location", DockerLocations.upload(context, name, uuid));
    error.getHeaders().replace("Range", rangeHeader(current));
    error.getHeaders().replace(DockerResponses.UPLOAD_UUID_HEADER, uuid);
    return error;
  }

  private Response finishBlob(final Context context, final String name, final String digest, final TempBlob blob) {
    String actual = DockerDigest.of(blob, digest);
    if (!digest.equals(actual)) {
      return DockerResponses.error(DockerError.DIGEST_INVALID,
          "uploaded content has digest " + actual + " instead of " + digest, Map.of("digest", digest));
    }
    content(context).saveBlob(blob, digest);
    return blobCreated(context, name, digest);
  }

  private static Response blobCreated(final Context context, final String name, final String digest) {
    return DockerResponses.success(201)
        .header("Location", DockerLocations.blob(context, name, digest))
        .header(DockerResponses.CONTENT_DIGEST_HEADER, digest)
        .header("Content-Length", "0")
        .build();
  }

  private long appendBody(final Context context, final Path path) throws IOException {
    Payload payload = context.getRequest().getPayload();
    if (payload == null) {
      return uploadStore.size(path);
    }
    try (InputStream in = payload.openInputStream()) {
      return uploadStore.append(path, in);
    }
  }

  private static Response uploadStatus(
      final Context context,
      final String name,
      final String uuid,
      final long size,
      final int status)
  {
    return DockerResponses.success(status)
        .header("Location", DockerLocations.upload(context, name, uuid))
        .header("Range", rangeHeader(size))
        .header(DockerResponses.UPLOAD_UUID_HEADER, uuid)
        .header("OCI-Chunk-Min-Length", "0")
        .header("Content-Length", "0")
        .build();
  }

  private static String rangeHeader(final long size) {
    return "0-" + Math.max(0, size - 1);
  }

  @Nullable
  static long[] parseContentRange(@Nullable final String header) {
    if (header == null || header.isBlank()) {
      return null;
    }
    String value = header.trim();
    if (value.startsWith("bytes")) {
      value = value.substring(5).trim();
      if (value.startsWith("=")) {
        value = value.substring(1).trim();
      }
    }
    int slash = value.indexOf('/');
    if (slash >= 0) {
      value = value.substring(0, slash);
    }
    int dash = value.indexOf('-');
    if (dash <= 0) {
      return null;
    }
    try {
      long start = Long.parseLong(value.substring(0, dash).trim());
      long end = Long.parseLong(value.substring(dash + 1).trim());
      return new long[]{start, end};
    }
    catch (NumberFormatException e) {
      log.debug("Ignoring malformed Content-Range {}", header);
      return null;
    }
  }
}
