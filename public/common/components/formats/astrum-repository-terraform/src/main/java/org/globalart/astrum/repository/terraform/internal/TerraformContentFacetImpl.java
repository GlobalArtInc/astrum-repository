package org.globalart.astrum.repository.terraform.internal;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;

import org.globalart.astrum.blobstore.api.Blob;
import org.globalart.astrum.common.hash.HashAlgorithm;
import org.globalart.astrum.repository.config.WritePolicy;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.facet.ContentFacetSupport;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.content.fluent.FluentQuery;
import org.globalart.astrum.repository.content.store.FormatStoreManager;
import org.globalart.astrum.repository.terraform.AssetKind;
import org.globalart.astrum.repository.terraform.ModuleCoordinates;
import org.globalart.astrum.repository.terraform.ProviderCoordinates;
import org.globalart.astrum.repository.terraform.TerraformContentFacet;
import org.globalart.astrum.repository.terraform.TerraformFormat;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.payloads.TempBlob;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.common.entity.Continuations.iterableOf;
import static org.globalart.astrum.common.hash.HashAlgorithm.SHA1;
import static org.globalart.astrum.common.hash.HashAlgorithm.SHA256;
import static org.globalart.astrum.repository.config.WritePolicy.ALLOW;
import static org.globalart.astrum.repository.config.WritePolicy.ALLOW_ONCE;

@Component
@Qualifier(TerraformFormat.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class TerraformContentFacetImpl
    extends ContentFacetSupport
    implements TerraformContentFacet
{
  public static final List<HashAlgorithm> HASHING = List.of(SHA1, SHA256);

  private static final String PREFIX_FILTER = "path LIKE #{filterParams.prefix}";

  private static final String KIND_PREFIX_FILTER = "kind = #{filterParams.kind} AND path LIKE #{filterParams.prefix}";

  @Autowired
  public TerraformContentFacetImpl(@Qualifier(TerraformFormat.NAME) final FormatStoreManager formatStoreManager) {
    super(formatStoreManager);
  }

  /**
   * Only module packages and provider packages are releases. Checksums, signatures and keys of a version are
   * regenerated as its packages change, and cached registry responses are refreshed, so they stay writable.
   */
  @Override
  protected WritePolicy writePolicy(final Asset asset) {
    WritePolicy writePolicy = super.writePolicy(asset);
    if (writePolicy == ALLOW_ONCE && !AssetKind.MODULE_ARCHIVE.name().equals(asset.kind())
        && !AssetKind.PROVIDER_ARCHIVE.name().equals(asset.kind())) {
      return ALLOW;
    }
    return writePolicy;
  }

  @Override
  protected String determineContentType(final Asset asset, final Blob blob, final boolean strictValidation) {
    String kind = asset.kind();
    if (AssetKind.MODULE_ARCHIVE.name().equals(kind) || AssetKind.PROVIDER_ARCHIVE.name().equals(kind)) {
      return super.determineContentType(asset, blob, strictValidation);
    }
    return TerraformContentTypes.forKind(kind);
  }

  @Override
  public Optional<FluentAsset> getAsset(final String path) {
    return assets().path(path).find();
  }

  @Override
  public Optional<Content> get(final String path) {
    return getAsset(path).map(FluentAsset::download);
  }

  @Override
  public TempBlob ingest(final Payload payload) {
    return blobs().ingest(payload, HASHING);
  }

  @Override
  public FluentAsset putModule(
      final ModuleCoordinates coordinates,
      final String path,
      final TempBlob blob,
      final Map<String, Object> attributes)
  {
    FluentComponent component = components()
        .name(coordinates.componentName())
        .namespace(coordinates.namespace())
        .version(coordinates.version())
        .kind(KIND_MODULE)
        .getOrCreate();
    Map<String, Object> values = new LinkedHashMap<>();
    values.put(KIND, KIND_MODULE);
    values.put(NAMESPACE, coordinates.namespace());
    values.put(NAME, coordinates.name());
    values.put(PROVIDER, coordinates.system());
    values.put(VERSION, coordinates.version());
    values.putAll(attributes);
    return assets()
        .path(path)
        .kind(AssetKind.MODULE_ARCHIVE.name())
        .component(component)
        .blob(blob)
        .save()
        .withAttribute(ATTRIBUTES_KEY, values);
  }

  @Override
  public FluentAsset putProviderFile(
      final ProviderCoordinates coordinates,
      final String fileName,
      final AssetKind kind,
      final TempBlob blob,
      final Map<String, Object> attributes)
  {
    FluentComponent component = components()
        .name(coordinates.componentName())
        .namespace(coordinates.namespace())
        .version(coordinates.version())
        .kind(KIND_PROVIDER)
        .getOrCreate();
    Map<String, Object> values = new LinkedHashMap<>();
    values.put(KIND, KIND_PROVIDER);
    values.put(NAMESPACE, coordinates.namespace());
    values.put(NAME, coordinates.type());
    values.put(PROVIDER, coordinates.type());
    values.put(VERSION, coordinates.version());
    values.putAll(attributes);
    return assets()
        .path(coordinates.filePath(fileName))
        .kind(kind.name())
        .component(component)
        .blob(blob)
        .save()
        .withAttribute(ATTRIBUTES_KEY, values);
  }

  @Override
  public FluentAsset putMetadata(final String path, final AssetKind kind, final TempBlob blob) {
    return assets()
        .path(path)
        .kind(kind.name())
        .blob(blob)
        .save();
  }

  @Override
  public boolean delete(final String path) {
    return getAsset(path).map(FluentAsset::delete).orElse(false);
  }

  @Override
  public List<FluentAsset> assetsIn(final String directory, final AssetKind... kinds) {
    String prefix = directory.endsWith("/") ? directory : directory + "/";
    return query(prefix, kinds).stream()
        .filter(asset -> asset.path().indexOf('/', prefix.length()) < 0)
        .toList();
  }

  @Override
  public List<FluentAsset> assetsUnder(final String directory, final AssetKind... kinds) {
    return query(directory.endsWith("/") ? directory : directory + "/", kinds);
  }

  private List<FluentAsset> query(final String prefix, final AssetKind... kinds) {
    String like = escapeLike(prefix) + "%";
    List<FluentAsset> found = new ArrayList<>();
    List<AssetKind> wanted = Arrays.asList(kinds);
    if (wanted.size() == 1) {
      collect(assets().byFilter(KIND_PREFIX_FILTER, Map.of("kind", wanted.get(0).name(), "prefix", like)), prefix,
          wanted, found);
    }
    else {
      collect(assets().byFilter(PREFIX_FILTER, Map.of("prefix", like)), prefix, wanted, found);
    }
    return found;
  }

  private static void collect(
      final FluentQuery<FluentAsset> query,
      final String prefix,
      final List<AssetKind> kinds,
      final List<FluentAsset> found)
  {
    for (FluentAsset asset : iterableOf(query::browse)) {
      if (asset.path().startsWith(prefix)
          && (kinds.isEmpty() || kinds.stream().anyMatch(kind -> kind.name().equals(asset.kind())))) {
        found.add(asset);
      }
    }
  }

  /**
   * Names may contain underscores, which are wildcards in LIKE patterns; matching paths are filtered again by
   * prefix, so a wildcard only costs a few extra rows.
   */
  private static String escapeLike(final String prefix) {
    return prefix.replace("%", "");
  }
}
