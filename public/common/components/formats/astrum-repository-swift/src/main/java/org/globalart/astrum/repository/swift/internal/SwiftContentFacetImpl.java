package org.globalart.astrum.repository.swift.internal;

import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.HashSet;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Optional;
import java.util.Set;

import javax.annotation.Nullable;

import org.globalart.astrum.blobstore.api.Blob;
import org.globalart.astrum.common.hash.HashAlgorithm;
import org.globalart.astrum.repository.config.WritePolicy;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.facet.ContentFacetSupport;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.content.store.FormatStoreManager;
import org.globalart.astrum.repository.content.store.InternalIds;
import org.globalart.astrum.repository.swift.AssetKind;
import org.globalart.astrum.repository.swift.SwiftCoordinates;
import org.globalart.astrum.repository.swift.SwiftFormat;
import org.globalart.astrum.repository.swift.SwiftVersion;
import org.globalart.astrum.repository.swift.internal.ReleaseMetadata.Signing;
import org.globalart.astrum.repository.swift.internal.store.SwiftComponentStore;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.payloads.TempBlob;
import org.globalart.astrum.security.ClientInfo;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.node.ObjectNode;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.nio.charset.StandardCharsets.UTF_8;
import static org.globalart.astrum.common.entity.Continuations.streamOf;
import static org.globalart.astrum.common.hash.HashAlgorithm.SHA1;
import static org.globalart.astrum.common.hash.HashAlgorithm.SHA256;
import static org.globalart.astrum.repository.content.AttributeOperation.SET;
import static org.globalart.astrum.repository.swift.internal.SwiftContentFacet.swiftAttributes;
import static org.globalart.astrum.repository.swift.internal.SwiftResponses.APPLICATION_JSON;
import static org.globalart.astrum.repository.swift.internal.SwiftResponses.APPLICATION_ZIP;
import static org.globalart.astrum.repository.swift.internal.SwiftResponses.TEXT_X_SWIFT;

@org.springframework.stereotype.Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
@Qualifier(SwiftFormat.NAME)
public class SwiftContentFacetImpl
    extends ContentFacetSupport
    implements SwiftContentFacet
{
  private static final List<HashAlgorithm> HASH_ALGORITHMS = List.of(SHA1, SHA256);

  private static final String EXACT_FILTER = "namespace = #{filterParams.scope} AND name = #{filterParams.name}";

  private static final String LOWER_FILTER =
      "LOWER(namespace) = #{filterParams.scope} AND LOWER(name) = #{filterParams.name}";

  @Autowired
  public SwiftContentFacetImpl(@Qualifier(SwiftFormat.NAME) final FormatStoreManager formatStoreManager) {
    super(formatStoreManager);
  }

  @Override
  public SwiftCoordinates canonical(final SwiftCoordinates coordinates) {
    return components(coordinates).stream()
        .findFirst()
        .map(component -> coordinates.withPackage(component.namespace(), component.name()))
        .orElse(coordinates);
  }

  @Override
  public List<FluentComponent> components(final SwiftCoordinates coordinates) {
    List<FluentComponent> exact = streamOf(components().byFilter(EXACT_FILTER,
        Map.of("scope", coordinates.getScope(), "name", coordinates.getName()))::browse).toList();
    if (!exact.isEmpty()) {
      return exact;
    }
    return streamOf(components().byFilter(LOWER_FILTER,
        Map.of("scope", coordinates.getLowerScope(), "name", coordinates.getLowerName()))::browse).toList();
  }

  @Override
  public Optional<FluentComponent> release(final SwiftCoordinates coordinates) {
    checkNotNull(coordinates.getVersion());
    Optional<FluentComponent> exact = components()
        .name(coordinates.getName())
        .namespace(coordinates.getScope())
        .version(coordinates.getVersion())
        .find();
    if (exact.isPresent()) {
      return exact;
    }
    return components(coordinates).stream()
        .filter(component -> coordinates.getVersion().equals(component.version()))
        .findFirst();
  }

  @Override
  public List<FluentComponent> releases(final SwiftCoordinates coordinates) {
    return components(coordinates).stream()
        .filter(component -> swiftAttributes(component).get(CHECKSUM) != null)
        .sorted(Comparator.comparing(FluentComponent::version, SwiftVersion.PRECEDENCE).reversed())
        .toList();
  }

  @Override
  public Optional<FluentAsset> getAsset(final String path) {
    return assets().path(checkNotNull(path)).find();
  }

  @Override
  public TempBlob ingest(final InputStream in, final String contentType) {
    return blobs().ingest(in, contentType, HASH_ALGORITHMS);
  }

  @Override
  public FluentComponent publish(
      final SwiftCoordinates requested,
      final TempBlob archive,
      @Nullable final byte[] metadataBytes,
      @Nullable final Signing signing,
      @Nullable final Signing metadataSigning) throws IOException
  {
    WritePolicy policy = getConfiguredWritePolicy();
    if (policy == WritePolicy.DENY) {
      throw SwiftProblem.methodNotAllowed("publishing isn't supported: repository " + repository().getName()
          + " is read-only");
    }
    ObjectNode metadata = ReleaseMetadata.parse(metadataBytes);
    SourceArchive source;
    try (InputStream in = archive.get()) {
      source = SourceArchive.read(in);
    }
    SwiftCoordinates coordinates = canonical(requested);
    Optional<FluentComponent> existing = release(coordinates);
    if (existing.isPresent() && swiftAttributes(existing.get()).get(CHECKSUM) != null && policy != WritePolicy.ALLOW) {
      throw SwiftProblem.conflict("a release with version " + coordinates.getVersion() + " already exists");
    }
    coordinates = existing.map(component -> requested.withPackage(component.namespace(), component.name()))
        .orElse(coordinates);

    FluentComponent component = components()
        .name(coordinates.getName())
        .namespace(coordinates.getScope())
        .version(coordinates.getVersion())
        .getOrCreate();
    assets()
        .path(coordinates.getArchivePath())
        .kind(AssetKind.ARCHIVE.name())
        .component(component)
        .blob(archive)
        .save();

    Map<String, String> toolsVersions = source.getToolsVersions();
    Map<String, String> alternates = new LinkedHashMap<>(toolsVersions);
    alternates.remove(Manifests.MANIFEST);
    Set<String> manifestPaths = new HashSet<>();
    for (Map.Entry<String, String> manifest : source.getManifests().entrySet()) {
      String filename = manifest.getKey();
      String path = coordinates.getManifestPath(filename);
      manifestPaths.add(path);
      Map<String, Object> attributes = new LinkedHashMap<>();
      attributes.put(TOOLS_VERSION, toolsVersions.get(filename));
      if (Manifests.MANIFEST.equals(filename)) {
        attributes.put(ALTERNATES, alternates);
      }
      try (TempBlob blob = ingest(new ByteArrayInputStream(manifest.getValue().getBytes(UTF_8)), TEXT_X_SWIFT)) {
        assets()
            .path(path)
            .kind(AssetKind.MANIFEST.name())
            .component(component)
            .blob(blob)
            .attributes(ATTRIBUTES, attributes)
            .save();
      }
    }
    component.assets().stream()
        .filter(asset -> AssetKind.MANIFEST.name().equals(asset.kind()))
        .filter(asset -> !manifestPaths.contains(asset.path()))
        .forEach(FluentAsset::delete);

    List<String> repositoryUrls = ReleaseMetadata.repositoryUrls(metadata);
    Map<String, Object> attributes = new LinkedHashMap<>();
    attributes.put(CHECKSUM, archive.getHashes().get(SHA256).toString());
    attributes.put(PUBLISHED_AT, ReleaseMetadata.now());
    attributes.put(METADATA, ReleaseMetadata.toMap(metadata));
    putIfPresent(attributes, DESCRIPTION, ReleaseMetadata.text(metadata, DESCRIPTION));
    if (!repositoryUrls.isEmpty()) {
      attributes.put(REPOSITORY_URLS, repositoryUrls);
    }
    if (signing != null) {
      attributes.put(SIGNING, signingMap(signing));
    }
    if (metadataSigning != null) {
      attributes.put(METADATA_SIGNING, signingMap(metadataSigning));
    }
    attributes.put(TOOLS_VERSION, toolsVersions.get(Manifests.MANIFEST));
    clientInfo().map(ClientInfo::getUserid).filter(Objects::nonNull).ifPresent(user -> attributes.put("publisher", user));
    component.attributes(SET, ATTRIBUTES, attributes);

    componentStore().replacePackageUrls(contentRepositoryId(), InternalIds.internalComponentId(component),
        PackageUrls.normalizeAll(repositoryUrls));
    return component;
  }

  @Override
  public List<String> identifiers(final String url) {
    String normalized = PackageUrls.normalize(url);
    if (normalized == null) {
      return List.of();
    }
    return componentStore().findPackageIdentifiers(contentRepositoryId(), normalized);
  }

  @Override
  public FluentAsset storeReleases(
      final SwiftCoordinates coordinates,
      final Content content,
      final List<Map<String, String>> locations)
  {
    SwiftCoordinates canonical = canonical(coordinates);
    try (TempBlob blob = blobs().ingest(content, HASH_ALGORITHMS)) {
      return assets()
          .path(canonical.getReleasesPath())
          .kind(AssetKind.RELEASES.name())
          .blob(blob)
          .attributes(ATTRIBUTES, Map.of(LOCATIONS, locations))
          .save();
    }
  }

  @Override
  public FluentAsset storeRelease(final SwiftCoordinates requested, final Content content) throws IOException {
    byte[] bytes = read(content, 4 * 1024 * 1024);
    JsonNode release = SwiftResponses.readQuietly(bytes);
    if (!(release instanceof ObjectNode) || !requested.getVersion().equals(ReleaseMetadata.text(release, "version"))) {
      throw new IOException("Remote returned invalid release metadata for " + requested);
    }
    SwiftCoordinates fromId = SwiftCoordinates.fromIdentifier(ReleaseMetadata.text(release, "id"),
        requested.getVersion());
    SwiftCoordinates coordinates = fromId != null && fromId.samePackage(requested.getScope(), requested.getName())
        ? canonicalOr(requested, fromId)
        : canonical(requested);

    FluentComponent component = components()
        .name(coordinates.getName())
        .namespace(coordinates.getScope())
        .version(coordinates.getVersion())
        .getOrCreate();
    FluentAsset asset;
    try (TempBlob blob = ingest(new ByteArrayInputStream(bytes), APPLICATION_JSON)) {
      asset = assets()
          .path(coordinates.getReleaseMetadataPath())
          .kind(AssetKind.RELEASE.name())
          .component(component)
          .blob(blob)
          .save();
    }
    JsonNode metadata = release.get(METADATA);
    Map<String, Object> attributes = new LinkedHashMap<>();
    putIfPresent(attributes, CHECKSUM, ReleaseMetadata.checksum(release));
    putIfPresent(attributes, PUBLISHED_AT, ReleaseMetadata.text(release, PUBLISHED_AT));
    if (metadata instanceof ObjectNode object) {
      attributes.put(METADATA, ReleaseMetadata.toMap(object));
      putIfPresent(attributes, DESCRIPTION, ReleaseMetadata.text(object, DESCRIPTION));
      List<String> urls = ReleaseMetadata.repositoryUrls(object);
      if (!urls.isEmpty()) {
        attributes.put(REPOSITORY_URLS, urls);
      }
    }
    Signing signing = ReleaseMetadata.signing(release);
    if (signing != null) {
      attributes.put(SIGNING, signingMap(signing));
    }
    component.attributes(SET, ATTRIBUTES, attributes);
    return asset;
  }

  @Override
  public FluentAsset storeManifest(
      final SwiftCoordinates requested,
      final String filename,
      final Content content,
      final Map<String, String> alternates) throws IOException
  {
    SwiftCoordinates coordinates = canonical(requested);
    byte[] bytes = read(content, SourceArchive.MAX_MANIFEST_SIZE);
    Map<String, Object> attributes = new LinkedHashMap<>();
    attributes.put(TOOLS_VERSION, Manifests.toolsVersionOrVariant(filename, new String(bytes, UTF_8)));
    if (Manifests.MANIFEST.equals(filename)) {
      attributes.put(ALTERNATES, alternates);
    }
    FluentComponent component = components()
        .name(coordinates.getName())
        .namespace(coordinates.getScope())
        .version(coordinates.getVersion())
        .getOrCreate();
    try (TempBlob blob = ingest(new ByteArrayInputStream(bytes), TEXT_X_SWIFT)) {
      return assets()
          .path(coordinates.getManifestPath(filename))
          .kind(AssetKind.MANIFEST.name())
          .component(component)
          .blob(blob)
          .attributes(ATTRIBUTES, attributes)
          .save();
    }
  }

  @Override
  public FluentAsset storeArchive(final SwiftCoordinates requested, final Content content) throws IOException {
    SwiftCoordinates coordinates = canonical(requested);
    FluentComponent component = components()
        .name(coordinates.getName())
        .namespace(coordinates.getScope())
        .version(coordinates.getVersion())
        .getOrCreate();
    try (TempBlob blob = blobs().ingest(content, HASH_ALGORITHMS)) {
      String checksum = blob.getHashes().get(SHA256).toString();
      Object expected = swiftAttributes(component).get(CHECKSUM);
      if (expected instanceof String value && !value.equalsIgnoreCase(checksum)) {
        throw new IOException("Source archive of " + coordinates + " has checksum " + checksum
            + " but the release metadata of the remote announces " + value);
      }
      FluentAsset asset = assets()
          .path(coordinates.getArchivePath())
          .kind(AssetKind.ARCHIVE.name())
          .component(component)
          .blob(blob)
          .save();
      if (expected == null) {
        Map<String, Object> attributes = new LinkedHashMap<>(swiftAttributes(component));
        attributes.put(CHECKSUM, checksum);
        component.attributes(SET, ATTRIBUTES, attributes);
      }
      return asset;
    }
  }

  @Override
  protected String determineContentType(final Asset asset, final Blob blob, final boolean strictValidation) {
    String path = asset.path();
    if (path.endsWith(SwiftCoordinates.JSON_EXTENSION)) {
      return APPLICATION_JSON;
    }
    if (path.endsWith(".swift")) {
      return TEXT_X_SWIFT;
    }
    String contentType = super.determineContentType(asset, blob, strictValidation);
    return path.endsWith(SwiftCoordinates.ARCHIVE_EXTENSION) ? APPLICATION_ZIP : contentType;
  }

  private SwiftCoordinates canonicalOr(final SwiftCoordinates requested, final SwiftCoordinates fallback) {
    List<FluentComponent> existing = components(requested);
    return existing.isEmpty()
        ? fallback
        : requested.withPackage(existing.get(0).namespace(), existing.get(0).name());
  }

  private SwiftComponentStore componentStore() {
    return (SwiftComponentStore) stores().componentStore;
  }

  private static Map<String, Object> signingMap(final Signing signing) {
    Map<String, Object> map = new LinkedHashMap<>();
    map.put("signatureBase64Encoded", signing.signatureBase64Encoded());
    map.put("signatureFormat", signing.signatureFormat());
    return map;
  }

  private static void putIfPresent(final Map<String, Object> map, final String key, @Nullable final Object value) {
    if (value != null) {
      map.put(key, value);
    }
  }

  static byte[] read(final Content content, final int limit) throws IOException {
    try (InputStream in = content.openInputStream()) {
      byte[] bytes = in.readNBytes(limit + 1);
      if (bytes.length > limit) {
        throw new IOException("Remote content is larger than " + limit + " bytes");
      }
      return bytes;
    }
  }

  static List<String> versions(final List<FluentComponent> components) {
    List<String> versions = new ArrayList<>();
    components.forEach(component -> versions.add(component.version()));
    return versions;
  }
}
