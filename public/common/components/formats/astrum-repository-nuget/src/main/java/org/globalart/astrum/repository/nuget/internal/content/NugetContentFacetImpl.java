package org.globalart.astrum.repository.nuget.internal.content;

import java.io.IOException;
import java.io.InputStream;
import java.time.OffsetDateTime;
import java.time.ZoneOffset;
import java.time.format.DateTimeFormatter;
import java.time.temporal.ChronoUnit;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Optional;

import javax.annotation.Nullable;

import org.globalart.astrum.common.entity.Continuations;
import org.globalart.astrum.common.hash.HashAlgorithm;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.facet.ContentFacetSupport;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.fluent.FluentAssetBuilder;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.content.maintenance.ContentMaintenanceFacet;
import org.globalart.astrum.repository.content.store.FormatStoreManager;
import org.globalart.astrum.repository.nuget.internal.AssetKind;
import org.globalart.astrum.repository.nuget.internal.NugetFormat;
import org.globalart.astrum.repository.nuget.internal.NugetPaths;
import org.globalart.astrum.repository.nuget.internal.metadata.InvalidNugetPackageException;
import org.globalart.astrum.repository.nuget.internal.metadata.NugetPackageMetadata;
import org.globalart.astrum.repository.nuget.internal.metadata.NugetVersion;
import org.globalart.astrum.repository.nuget.internal.metadata.NuspecParser;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.payloads.TempBlob;

import com.google.common.collect.ImmutableList;
import com.google.common.hash.HashCode;
import com.google.common.io.BaseEncoding;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;

import static org.globalart.astrum.common.hash.HashAlgorithm.MD5;
import static org.globalart.astrum.common.hash.HashAlgorithm.SHA1;
import static org.globalart.astrum.common.hash.HashAlgorithm.SHA256;
import static org.globalart.astrum.common.hash.HashAlgorithm.SHA512;
import static org.globalart.astrum.repository.nuget.internal.NugetPaths.lower;

@org.springframework.stereotype.Component
@Qualifier(NugetFormat.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class NugetContentFacetImpl
    extends ContentFacetSupport
    implements NugetContentFacet
{
  private static final Iterable<HashAlgorithm> HASHING = ImmutableList.of(MD5, SHA1, SHA256, SHA512);

  @Autowired
  public NugetContentFacetImpl(@Qualifier(NugetFormat.NAME) final FormatStoreManager formatStoreManager) {
    super(formatStoreManager);
  }

  @Override
  public Optional<Content> get(final String path) {
    return findAsset(path).map(FluentAsset::download);
  }

  @Override
  public Optional<FluentAsset> findAsset(final String path) {
    return assets().path(path).find();
  }

  @Override
  public TempBlob ingest(final Payload payload) {
    return blobs().ingest(payload, HASHING);
  }

  @Override
  public Content putPackage(
      final TempBlob blob,
      final NugetPackageMetadata metadata,
      @Nullable final Payload original)
  {
    String version = metadata.getVersion().toNormalizedString();
    return savePackage(NugetPaths.packagePath(metadata.getId(), version), blob, metadata, original);
  }

  @Override
  public Content putPackage(final String path, final Payload payload) throws IOException {
    try (TempBlob blob = ingest(payload)) {
      NugetPackageMetadata metadata;
      try (InputStream in = blob.get()) {
        metadata = NuspecParser.parsePackage(in);
      }
      catch (InvalidNugetPackageException | IOException e) {
        log.warn("Unable to read package metadata of {} in {}: {}", path, getRepository().getName(), e.getMessage());
        FluentAsset asset = assets().path(path).kind(AssetKind.PACKAGE.name()).blob(blob).save();
        asset.markAsCached(payload);
        return asset.download();
      }
      return savePackage(path, blob, metadata, payload);
    }
  }

  private Content savePackage(
      final String path,
      final TempBlob blob,
      final NugetPackageMetadata metadata,
      @Nullable final Payload original)
  {
    findAsset(path).ifPresent(this::checkAttachAllowed);

    metadata.setText("published", OffsetDateTime.now(ZoneOffset.UTC).truncatedTo(ChronoUnit.MILLIS)
        .format(DateTimeFormatter.ISO_OFFSET_DATE_TIME));
    metadata.setPackageSize(blob.getBlob().getMetrics().getContentSize());
    HashCode sha512 = blob.getHashes().get(SHA512);
    if (sha512 != null) {
      metadata.setText("package_hash", BaseEncoding.base64().encode(sha512.asBytes()));
      metadata.setText("package_hash_algorithm", "SHA512");
    }

    String version = metadata.getVersion().toNormalizedString();
    Map<String, Object> componentAttributes = metadata.toAttributes();
    Optional<FluentComponent> existing = findComponent(metadata.getId(), version);
    FluentComponent component = existing
        .map(found -> found.withAttribute(NugetFormat.NAME, componentAttributes))
        .orElseGet(() -> components()
            .name(canonicalName(metadata.getId()))
            .version(version)
            .attributes(NugetFormat.NAME, componentAttributes)
            .getOrCreate());

    Map<String, Object> assetAttributes = Map.of(NugetPackageMetadata.ID, metadata.getId(),
        NugetPackageMetadata.VERSION, metadata.getVersion().toFullString(),
        NugetPackageMetadata.TAGS, metadata.text(NugetPackageMetadata.TAGS));
    FluentAssetBuilder builder = assets().path(path).kind(AssetKind.PACKAGE.name()).component(component)
        .attributes(NugetFormat.NAME, assetAttributes)
        .blob(blob);
    FluentAsset asset = builder.save();
    if (original != null) {
      asset.markAsCached(original);
    }
    return asset.download();
  }

  private String canonicalName(final String id) {
    return findComponents(id).stream().map(Component::name).findFirst().orElse(id);
  }

  @Override
  public Content putMetadata(final String path, final Payload payload, final AssetKind kind) throws IOException {
    try (TempBlob blob = ingest(payload)) {
      FluentAsset asset = assets().path(path).kind(kind.name()).blob(blob).save();
      asset.markAsCached(payload);
      return asset.download();
    }
  }

  @Override
  public List<FluentComponent> findComponents(final String id) {
    return Continuations.streamOf(components()
            .byFilter("LOWER(name) = #{filterParams.lowerName}", Map.of("lowerName", lower(id)))::browse)
        .toList();
  }

  @Override
  public List<NugetPackageMetadata> findPackages(final String id) {
    List<NugetPackageMetadata> result = new ArrayList<>();
    for (FluentComponent component : findComponents(id)) {
      metadata(component).ifPresent(result::add);
    }
    return result;
  }

  @Override
  public Optional<FluentComponent> findComponent(final String id, final String version) {
    Optional<NugetVersion> requested = NugetVersion.parse(version);
    return findComponents(id).stream()
        .filter(component -> requested.isPresent()
            ? NugetVersion.parse(component.version()).map(requested.get()::equals).orElse(false)
            : lower(component.version()).equals(lower(version)))
        .findFirst();
  }

  @Override
  public List<NugetPackageMetadata> allPackages() {
    List<NugetPackageMetadata> result = new ArrayList<>();
    Continuations.streamOf(components()::browse).forEach(component -> metadata(component).ifPresent(result::add));
    return result;
  }

  public static Optional<NugetPackageMetadata> metadata(final Component component) {
    Object attributes = component.attributes().get(NugetFormat.NAME);
    if (attributes instanceof Map<?, ?> map) {
      @SuppressWarnings("unchecked")
      Map<String, Object> typed = (Map<String, Object>) map;
      return Optional.ofNullable(NugetPackageMetadata.fromAttributes(typed));
    }
    return Optional.empty();
  }

  @Override
  public boolean deletePackage(final String id, final String version) {
    Optional<FluentComponent> component = findComponent(id, version);
    if (component.isEmpty()) {
      return false;
    }
    repository().facet(ContentMaintenanceFacet.class).deleteComponent(component.get());
    return true;
  }

  @Override
  public Optional<byte[]> nuspec(final String id, final String version) throws IOException {
    String normalized = NugetVersion.normalize(version).orElse(version);
    Optional<Content> content = get(NugetPaths.packagePath(id, normalized));
    if (content.isEmpty()) {
      return Optional.empty();
    }
    try (InputStream in = content.get().openInputStream()) {
      return NuspecParser.readNuspec(in);
    }
  }
}
