package org.globalart.astrum.repository.docker.internal;

import java.io.ByteArrayInputStream;
import java.io.InputStream;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Optional;
import java.util.stream.Collectors;
import java.util.stream.Stream;

import javax.annotation.Nullable;

import org.globalart.astrum.blobstore.api.Blob;
import org.globalart.astrum.common.entity.Continuations;
import org.globalart.astrum.common.hash.HashAlgorithm;
import org.globalart.astrum.repository.config.ConfigurationConstants;
import org.globalart.astrum.repository.config.WritePolicy;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.facet.ContentFacetSupport;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.content.fluent.FluentQuery;
import org.globalart.astrum.repository.content.store.FormatStoreManager;
import org.globalart.astrum.repository.docker.DockerConfig;
import org.globalart.astrum.repository.docker.DockerContentFacet;
import org.globalart.astrum.repository.docker.DockerFormat;
import org.globalart.astrum.repository.docker.internal.v2.DockerManifest;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.payloads.TempBlob;

import com.google.common.collect.ImmutableList;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.blobstore.api.BlobStore.CONTENT_TYPE_HEADER;
import static org.globalart.astrum.common.hash.HashAlgorithm.SHA1;
import static org.globalart.astrum.common.hash.HashAlgorithm.SHA256;

@Component
@Qualifier(DockerFormat.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class DockerContentFacetImpl
    extends ContentFacetSupport
    implements DockerContentFacet
{
  public static final List<HashAlgorithm> HASHES = ImmutableList.of(SHA1, SHA256);

  private static final String OCTET_STREAM = "application/octet-stream";

  private static final String NAMESPACE = "";

  @Autowired
  public DockerContentFacetImpl(@Qualifier(DockerFormat.NAME) final FormatStoreManager formatStoreManager) {
    super(formatStoreManager);
  }

  @Override
  protected WritePolicy writePolicy(final Asset asset) {
    WritePolicy policy = super.writePolicy(asset);
    if (KIND_BLOB.equals(asset.kind()) || KIND_MANIFEST.equals(asset.kind())) {
      return policy.checkCreateAllowed() ? WritePolicy.ALLOW : policy;
    }
    if (policy == WritePolicy.ALLOW_ONCE && asset.path().endsWith("/manifests/" + LATEST) && latestPolicy()) {
      return WritePolicy.ALLOW;
    }
    return policy;
  }

  @Override
  protected String determineContentType(final Asset asset, final Blob blob, final boolean strictValidation) {
    String declared = blob.getHeaders().get(CONTENT_TYPE_HEADER);
    return declared == null || declared.isBlank() ? OCTET_STREAM : declared;
  }

  @Override
  public WritePolicy configuredWritePolicy() {
    return getConfiguredWritePolicy();
  }

  @Override
  public boolean latestPolicy() {
    return DockerConfig.bool(
        getRepository().getConfiguration().attributes(ConfigurationConstants.STORAGE)
            .get(ConfigurationConstants.LATEST_POLICY), false);
  }

  @Override
  public TempBlob ingest(final Payload payload) {
    return blobs().ingest(payload, HASHES);
  }

  @Override
  public TempBlob ingest(final InputStream in, @Nullable final String contentType) {
    return blobs().ingest(in, contentType, HASHES);
  }

  @Override
  public TempBlob ingest(final Path file) {
    return blobs().ingest(file, OCTET_STREAM, HASHES, false);
  }

  @Override
  public Optional<FluentAsset> findBlob(final String digest) {
    return assets().path(DockerContentFacet.blobPath(digest)).find().filter(FluentAsset::hasBlob);
  }

  @Override
  public FluentAsset saveBlob(final TempBlob blob, final String digest) {
    Optional<FluentAsset> existing = findBlob(digest);
    if (existing.isPresent()) {
      return existing.get();
    }
    return assets()
        .path(DockerContentFacet.blobPath(digest))
        .kind(KIND_BLOB)
        .attributes(DockerFormat.NAME, Map.of(ATTR_DIGEST, digest))
        .blob(blob)
        .save();
  }

  @Override
  public boolean deleteBlob(final String digest) {
    Optional<FluentAsset> asset = assets().path(DockerContentFacet.blobPath(digest)).find();
    return asset.isPresent() && asset.get().delete();
  }

  @Override
  public Optional<FluentAsset> findManifest(final String name, final String reference) {
    return assets().path(DockerContentFacet.manifestPath(name, reference)).find().filter(FluentAsset::hasBlob);
  }

  @Override
  public FluentAsset saveManifest(
      final String name,
      final String digest,
      final byte[] bytes,
      final DockerManifest manifest)
  {
    Optional<FluentAsset> existing = findManifest(name, digest);
    if (existing.isPresent()) {
      return existing.get();
    }
    try (TempBlob blob = ingest(new ByteArrayInputStream(bytes), manifest.getMediaType())) {
      return assets()
          .path(DockerContentFacet.manifestPath(name, digest))
          .kind(KIND_MANIFEST)
          .attributes(DockerFormat.NAME, manifestAttributes(name, digest, manifest))
          .blob(blob)
          .save();
    }
  }

  @Override
  public FluentAsset saveTag(
      final String name,
      final String tag,
      final String digest,
      final byte[] bytes,
      final DockerManifest manifest)
  {
    Optional<FluentAsset> existing = findManifest(name, tag);
    if (existing.isPresent() && digest.equals(DockerContentFacet.digestOf(existing.get()))) {
      return existing.get();
    }
    FluentComponent component = components()
        .name(name)
        .namespace(NAMESPACE)
        .version(tag)
        .getOrCreate();
    try (TempBlob blob = ingest(new ByteArrayInputStream(bytes), manifest.getMediaType())) {
      return assets()
          .path(DockerContentFacet.manifestPath(name, tag))
          .kind(KIND_TAG)
          .component(component)
          .attributes(DockerFormat.NAME, manifestAttributes(name, digest, manifest))
          .blob(blob)
          .save();
    }
  }

  private static Map<String, Object> manifestAttributes(
      final String name,
      final String digest,
      final DockerManifest manifest)
  {
    Map<String, Object> attributes = new LinkedHashMap<>();
    attributes.put(ATTR_NAME, name);
    attributes.put(ATTR_DIGEST, digest);
    attributes.put(ATTR_MEDIA_TYPE, manifest.getMediaType());
    if (manifest.getSubject() != null) {
      attributes.put(ATTR_SUBJECT, manifest.getSubject());
    }
    if (manifest.getArtifactType() != null) {
      attributes.put(ATTR_ARTIFACT_TYPE, manifest.getArtifactType());
    }
    return attributes;
  }

  @Override
  public boolean deleteTag(final String name, final String tag) {
    Optional<FluentAsset> asset = assets().path(DockerContentFacet.manifestPath(name, tag)).find();
    if (asset.isEmpty() || !KIND_TAG.equals(asset.get().kind())) {
      return false;
    }
    asset.get().delete();
    components().name(name).namespace(NAMESPACE).version(tag).find()
        .filter(component -> component.assets().isEmpty())
        .ifPresent(FluentComponent::delete);
    return true;
  }

  @Override
  public boolean deleteManifest(final String name, final String digest) {
    Optional<FluentAsset> asset = assets().path(DockerContentFacet.manifestPath(name, digest)).find();
    if (asset.isEmpty()) {
      return false;
    }
    for (FluentAsset tag : tagsByDigest(name, digest)) {
      String path = tag.path();
      deleteTag(name, path.substring(path.lastIndexOf('/') + 1));
    }
    return asset.get().delete();
  }

  @Override
  public List<FluentAsset> tagsByDigest(final String name, final String digest) {
    return manifestAssets(name, KIND_TAG)
        .filter(asset -> digest.equals(DockerContentFacet.digestOf(asset)))
        .collect(Collectors.toList());
  }

  @Override
  public List<FluentAsset> referrers(final String name, final String digest) {
    return manifestAssets(name, KIND_MANIFEST)
        .filter(asset -> digest.equals(asset.attributes(DockerFormat.NAME).get(ATTR_SUBJECT)))
        .collect(Collectors.toList());
  }

  private Stream<FluentAsset> manifestAssets(final String name, final String kind) {
    String prefix = DockerContentFacet.manifestsPrefix(name);
    FluentQuery<FluentAsset> query = assets().byFilter(
        "kind = #{filterParams.kind} AND path LIKE #{filterParams.prefix}",
        Map.of("kind", kind, "prefix", prefix + "%"));
    return Continuations.streamOf(query::browse)
        .filter(asset -> asset.path().startsWith(prefix) && asset.path().indexOf('/', prefix.length()) < 0);
  }

  @Override
  public List<String> tags(final String name) {
    List<String> tags = new ArrayList<>(components().versions(NAMESPACE, name));
    tags.removeIf(Objects::isNull);
    tags.sort(null);
    return tags;
  }

  @Override
  public List<String> names() {
    List<String> names = new ArrayList<>(components().names(NAMESPACE));
    names.sort(null);
    return names;
  }

  @Override
  public boolean hasName(final String name) {
    if (!components().versions(NAMESPACE, name).isEmpty()) {
      return true;
    }
    return manifestAssets(name, KIND_MANIFEST).findAny().isPresent();
  }
}
