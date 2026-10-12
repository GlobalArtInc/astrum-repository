package org.globalart.astrum.repository.docker;

import java.io.InputStream;
import java.nio.file.Path;
import java.util.List;
import java.util.Map;
import java.util.Optional;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.config.WritePolicy;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.docker.internal.v2.DockerManifest;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.payloads.TempBlob;

@Facet.Exposed
public interface DockerContentFacet
    extends ContentFacet
{
  String KIND_BLOB = "BLOB";

  String KIND_MANIFEST = "MANIFEST";

  String KIND_TAG = "TAG";

  String ATTR_DIGEST = "digest";

  String ATTR_MEDIA_TYPE = "mediaType";

  String ATTR_NAME = "name";

  String ATTR_SUBJECT = "subject";

  String ATTR_ARTIFACT_TYPE = "artifactType";

  String ATTR_ARTIFACT_KIND = "artifactKind";

  String ATTR_ANNOTATIONS = "annotations";

  String ATTR_MANIFESTS = "manifests";

  String BLOBS_PREFIX = "/v2/-/blobs/";

  String LATEST = "latest";

  static String blobPath(final String digest) {
    return BLOBS_PREFIX + digest;
  }

  static String manifestsPrefix(final String name) {
    return "/v2/" + name + "/manifests/";
  }

  static String manifestPath(final String name, final String reference) {
    return manifestsPrefix(name) + reference;
  }

  TempBlob ingest(Payload payload);

  TempBlob ingest(Payload payload, String digest);

  TempBlob ingest(InputStream in, @Nullable String contentType);

  TempBlob ingest(Path file);

  TempBlob ingest(Path file, String digest);

  Optional<FluentAsset> findBlob(String digest);

  FluentAsset saveBlob(TempBlob blob, String digest);

  boolean deleteBlob(String digest);

  Optional<FluentAsset> findManifest(String name, String reference);

  FluentAsset saveManifest(String name, String digest, byte[] bytes, DockerManifest manifest);

  FluentAsset saveTag(String name, String tag, String digest, byte[] bytes, DockerManifest manifest);

  boolean deleteTag(String name, String tag);

  /**
   * Deletes the manifest stored under the digest with the tags that point at it, and everything attached to it:
   * the manifests whose {@code subject} it is and the tags of the tag schema derived from its digest, recursively.
   */
  boolean deleteManifest(String name, String digest);

  List<FluentAsset> tagsByDigest(String name, String digest);

  /**
   * The manifests of the name whose {@code subject} is the digest.
   */
  List<FluentAsset> referrers(String name, String digest);

  /**
   * The descriptors of an index stored under the tag a client without the referrers API keeps for the digest.
   */
  List<Map<String, Object>> fallbackReferrers(String name, String digest);

  List<String> tags(String name);

  List<String> names();

  boolean hasName(String name);

  WritePolicy configuredWritePolicy();

  boolean latestPolicy();

  @Nullable
  static String digestOf(final FluentAsset asset) {
    Object digest = asset.attributes(DockerFormat.NAME).get(ATTR_DIGEST);
    return digest == null ? null : digest.toString();
  }

  @Nullable
  static String nameOf(final FluentAsset asset) {
    Object name = asset.attributes(DockerFormat.NAME).get(ATTR_NAME);
    return name == null ? null : name.toString();
  }

  @Nullable
  static String mediaTypeOf(final FluentAsset asset) {
    Object mediaType = asset.attributes(DockerFormat.NAME).get(ATTR_MEDIA_TYPE);
    if (mediaType != null) {
      return mediaType.toString();
    }
    return asset.blob().map(blob -> blob.contentType()).orElse(null);
  }
}
