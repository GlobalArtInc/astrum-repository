package org.globalart.astrum.repository.docker.internal.v2;

import java.io.IOException;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import java.util.Locale;
import java.util.Set;

import javax.annotation.Nullable;

import com.fasterxml.jackson.databind.JsonNode;
import com.google.common.collect.ImmutableSet;

public class DockerManifest
{
  public static final String DOCKER_V2 = "application/vnd.docker.distribution.manifest.v2+json";

  public static final String DOCKER_LIST = "application/vnd.docker.distribution.manifest.list.v2+json";

  public static final String DOCKER_V1 = "application/vnd.docker.distribution.manifest.v1+json";

  public static final String DOCKER_V1_SIGNED = "application/vnd.docker.distribution.manifest.v1+prettyjws";

  public static final String OCI_MANIFEST = "application/vnd.oci.image.manifest.v1+json";

  public static final String OCI_INDEX = "application/vnd.oci.image.index.v1+json";

  public static final String ACCEPT = String.join(", ", OCI_INDEX, DOCKER_LIST, OCI_MANIFEST, DOCKER_V2,
      DOCKER_V1_SIGNED, DOCKER_V1);

  private static final Set<String> INDEX_TYPES = ImmutableSet.of(OCI_INDEX, DOCKER_LIST);

  private static final Set<String> GENERIC_TYPES = ImmutableSet.of("application/json", "application/octet-stream",
      "text/plain");

  private final String mediaType;

  private final boolean index;

  private final boolean schema1;

  private final List<String> blobs;

  private final List<String> manifests;

  private final String subject;

  private final String artifactType;

  private DockerManifest(
      final String mediaType,
      final boolean index,
      final boolean schema1,
      final List<String> blobs,
      final List<String> manifests,
      @Nullable final String subject,
      @Nullable final String artifactType)
  {
    this.mediaType = mediaType;
    this.index = index;
    this.schema1 = schema1;
    this.blobs = blobs;
    this.manifests = manifests;
    this.subject = subject;
    this.artifactType = artifactType;
  }

  public String getMediaType() {
    return mediaType;
  }

  public boolean isIndex() {
    return index;
  }

  public boolean isSchema1() {
    return schema1;
  }

  public List<String> getBlobs() {
    return blobs;
  }

  public List<String> getManifests() {
    return manifests;
  }

  @Nullable
  public String getSubject() {
    return subject;
  }

  @Nullable
  public String getArtifactType() {
    return artifactType;
  }

  public static DockerManifest parse(final byte[] bytes, @Nullable final String contentType)
      throws InvalidManifestException
  {
    JsonNode root;
    try {
      root = DockerResponses.MAPPER.readTree(bytes);
    }
    catch (IOException e) {
      throw new InvalidManifestException("manifest is not valid JSON");
    }
    if (root == null || !root.isObject()) {
      throw new InvalidManifestException("manifest is not a JSON object");
    }

    int schemaVersion = root.path("schemaVersion").asInt(0);
    if (schemaVersion == 1) {
      List<String> blobs = new ArrayList<>();
      for (JsonNode layer : root.path("fsLayers")) {
        blobs.add(requireDigest(layer.path("blobSum")));
      }
      String type = root.has("signatures") ? DOCKER_V1_SIGNED : DOCKER_V1;
      return new DockerManifest(type, false, true, blobs, Collections.emptyList(), null, null);
    }
    if (schemaVersion != 2) {
      throw new InvalidManifestException("unsupported manifest schemaVersion " + schemaVersion);
    }

    String jsonType = text(root.path("mediaType"));
    String headerType = normalizeContentType(contentType);
    if (headerType != null && GENERIC_TYPES.contains(headerType)) {
      headerType = null;
    }
    if (jsonType != null && headerType != null && !jsonType.equals(headerType)) {
      throw new InvalidManifestException(
          "manifest mediaType " + jsonType + " does not match Content-Type " + headerType);
    }

    String type = jsonType != null ? jsonType : headerType;
    boolean isIndex = root.has("manifests") || (type != null && INDEX_TYPES.contains(type));
    if (type == null) {
      type = isIndex ? OCI_INDEX : OCI_MANIFEST;
    }

    List<String> blobs = new ArrayList<>();
    List<String> manifests = new ArrayList<>();
    if (isIndex) {
      for (JsonNode child : root.path("manifests")) {
        manifests.add(requireDigest(child.path("digest")));
      }
    }
    else {
      JsonNode config = root.path("config");
      if (!config.isMissingNode()) {
        blobs.add(requireDigest(config.path("digest")));
      }
      for (JsonNode layer : root.path("layers")) {
        String digest = requireDigest(layer.path("digest"));
        if (!isForeign(layer)) {
          blobs.add(digest);
        }
      }
    }

    String subject = null;
    JsonNode subjectNode = root.path("subject");
    if (subjectNode.isObject()) {
      subject = requireDigest(subjectNode.path("digest"));
    }

    String artifactType = text(root.path("artifactType"));
    if (artifactType == null && !isIndex) {
      artifactType = text(root.path("config").path("mediaType"));
    }

    return new DockerManifest(type, isIndex, false, blobs, manifests, subject, artifactType);
  }

  private static boolean isForeign(final JsonNode layer) {
    if (layer.path("urls").isArray() && layer.path("urls").size() > 0) {
      return true;
    }
    String type = text(layer.path("mediaType"));
    return type != null && (type.contains("foreign") || type.contains("nondistributable"));
  }

  private static String requireDigest(final JsonNode node) throws InvalidManifestException {
    String value = text(node);
    if (!DockerDigest.isDigest(value)) {
      throw new InvalidManifestException("invalid digest " + value);
    }
    return value;
  }

  @Nullable
  private static String text(final JsonNode node) {
    if (node == null || !node.isTextual()) {
      return null;
    }
    String value = node.asText();
    return value.isBlank() ? null : value;
  }

  @Nullable
  public static String normalizeContentType(@Nullable final String contentType) {
    if (contentType == null) {
      return null;
    }
    int semicolon = contentType.indexOf(';');
    String type = (semicolon < 0 ? contentType : contentType.substring(0, semicolon)).trim()
        .toLowerCase(Locale.ENGLISH);
    return type.isEmpty() ? null : type;
  }

  public static class InvalidManifestException
      extends Exception
  {
    public InvalidManifestException(final String message) {
      super(message);
    }
  }
}
