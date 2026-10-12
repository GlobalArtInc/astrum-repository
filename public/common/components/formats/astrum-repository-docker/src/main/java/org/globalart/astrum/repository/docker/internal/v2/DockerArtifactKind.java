package org.globalart.astrum.repository.docker.internal.v2;

import java.util.ArrayList;
import java.util.List;
import java.util.Locale;
import java.util.Set;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

import javax.annotation.Nullable;

/**
 * What a manifest carries, read from its artifact type, config and layer media types: a container image, a Helm
 * chart, a signature, an SBOM, an attestation, a Wasm module, an AI model, or some other OCI artifact.
 *
 * Signatures, SBOMs and attestations that a client stores under the tags of the tag schema ({@code sha256-<hex>.sig},
 * {@code .sbom}, {@code .att}) are recognised by the tag too; the referrers index a client keeps under
 * {@code sha256-<hex>} when a registry has no referrers API is an artifact of its own.
 */
public enum DockerArtifactKind
{
  IMAGE("image"),
  HELM("helm"),
  SIGNATURE("signature"),
  SBOM("sbom"),
  ATTESTATION("attestation"),
  WASM("wasm"),
  MODEL("model"),
  OTHER("other");

  public static final String OCI_IMAGE_CONFIG = "application/vnd.oci.image.config.v1+json";

  public static final String DOCKER_IMAGE_CONFIG = "application/vnd.docker.container.image.v1+json";

  public static final String HELM_CONFIG = "application/vnd.cncf.helm.config.v1+json";

  private static final Pattern TAG_SCHEMA = Pattern.compile("^sha256-[a-f0-9]{64}(?:\\.([a-z]+))?$");

  private static final Set<String> IMAGE_CONFIGS = Set.of(OCI_IMAGE_CONFIG, DOCKER_IMAGE_CONFIG);

  private static final Set<String> SIGNATURES = Set.of(
      "application/vnd.dev.cosign.artifact.sig.v1+json",
      "application/vnd.dev.cosign.simplesigning.v1+json",
      "application/vnd.cncf.notary.signature",
      "application/vnd.cncf.notary.v2.signature",
      "application/pgp-signature");

  private static final Set<String> ATTESTATIONS = Set.of(
      "application/vnd.dsse.envelope.v1+json",
      "application/vnd.in-toto+json",
      "application/vnd.dev.cosign.attestation.v1+json");

  private static final Set<String> SBOMS = Set.of(
      "application/vnd.dev.cosign.artifact.sbom.v1+json",
      "application/vnd.syft+json",
      "text/spdx",
      "text/spdx+json");

  private final String id;

  DockerArtifactKind(final String id) {
    this.id = id;
  }

  public String id() {
    return id;
  }

  @Nullable
  public static DockerArtifactKind fromId(@Nullable final String id) {
    for (DockerArtifactKind kind : values()) {
      if (kind.id.equals(id)) {
        return kind;
      }
    }
    return null;
  }

  public static DockerArtifactKind of(final DockerManifest manifest) {
    return of(manifest, null);
  }

  public static DockerArtifactKind of(final DockerManifest manifest, @Nullable final String tag) {
    List<String> types = new ArrayList<>();
    add(types, manifest.getExplicitArtifactType());
    add(types, manifest.getConfigMediaType());
    for (String layer : manifest.getLayerMediaTypes()) {
      add(types, layer);
    }

    DockerArtifactKind byType = byTypes(types);
    if (byType != null) {
      return byType;
    }
    DockerArtifactKind byTag = byTag(tag);
    if (byTag != null) {
      return byTag;
    }
    if (manifest.getExplicitArtifactType() != null) {
      return OTHER;
    }
    if (manifest.isIndex()) {
      return IMAGE;
    }
    String config = manifest.getConfigMediaType();
    if (config == null || IMAGE_CONFIGS.contains(config) || manifest.isSchema1()) {
      return IMAGE;
    }
    return OTHER;
  }

  @Nullable
  private static DockerArtifactKind byTypes(final List<String> types) {
    for (String type : types) {
      for (DockerArtifactKind kind : List.of(SIGNATURE, ATTESTATION, SBOM, HELM, WASM, MODEL)) {
        if (kind.matches(type)) {
          return kind;
        }
      }
    }
    return null;
  }

  private boolean matches(final String type) {
    switch (this) {
      case SIGNATURE:
        return SIGNATURES.contains(type) || type.startsWith("application/vnd.dev.sigstore.bundle");
      case ATTESTATION:
        return ATTESTATIONS.contains(type) || type.startsWith("application/vnd.in-toto.");
      case SBOM:
        return SBOMS.contains(type) || type.contains("spdx") || type.contains("cyclonedx");
      case HELM:
        return type.startsWith("application/vnd.cncf.helm.");
      case WASM:
        return type.contains("wasm");
      case MODEL:
        return type.startsWith("application/vnd.cncf.model.") || type.startsWith("application/vnd.docker.ai.")
            || type.startsWith("application/vnd.ollama.image.");
      default:
        return false;
    }
  }

  @Nullable
  static DockerArtifactKind byTag(@Nullable final String tag) {
    if (tag == null) {
      return null;
    }
    Matcher matcher = TAG_SCHEMA.matcher(tag);
    if (!matcher.matches()) {
      return null;
    }
    String suffix = matcher.group(1);
    if (suffix == null) {
      return OTHER;
    }
    switch (suffix) {
      case "sig":
        return SIGNATURE;
      case "att":
        return ATTESTATION;
      case "sbom":
        return SBOM;
      default:
        return OTHER;
    }
  }

  /**
   * The digest a tag of the tag schema belongs to ({@code sha256-<hex>.sig} belongs to {@code sha256:<hex>}), or
   * {@code null} for any other tag.
   */
  @Nullable
  public static String tagSchemaSubject(@Nullable final String tag) {
    if (tag == null || !TAG_SCHEMA.matcher(tag).matches()) {
      return null;
    }
    return "sha256:" + tag.substring(7, 71);
  }

  /**
   * The tag under which a client without the referrers API keeps the referrers index of a digest.
   */
  @Nullable
  public static String fallbackTag(final String digest) {
    if (!DockerDigest.isSupported(digest)) {
      return null;
    }
    String tag = digest.replace(':', '-');
    return tag.length() > 128 ? tag.substring(0, 128) : tag;
  }

  private static void add(final List<String> types, @Nullable final String type) {
    String normalized = DockerManifest.normalizeContentType(type);
    if (normalized != null) {
      types.add(normalized.toLowerCase(Locale.ENGLISH));
    }
  }
}
