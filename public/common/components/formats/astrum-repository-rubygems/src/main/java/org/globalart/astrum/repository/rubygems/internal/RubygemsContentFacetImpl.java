package org.globalart.astrum.repository.rubygems.internal;

import java.io.InputStream;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Optional;
import java.util.stream.Stream;

import javax.annotation.Nullable;

import org.globalart.astrum.blobstore.api.Blob;
import org.globalart.astrum.repository.config.WritePolicy;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.facet.ContentFacetSupport;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.content.store.FormatStoreManager;
import org.globalart.astrum.repository.rubygems.AssetKind;
import org.globalart.astrum.repository.rubygems.RubygemsFormat;
import org.globalart.astrum.repository.rubygems.internal.gem.GemSpec;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.payloads.TempBlob;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.entity.Continuations.streamOf;
import static org.globalart.astrum.repository.view.ContentTypes.TEXT_PLAIN;

@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
@Qualifier(RubygemsFormat.NAME)
public class RubygemsContentFacetImpl
    extends ContentFacetSupport
    implements RubygemsContentFacet
{
  private static final String NAME_FILTER = "name = #{filterParams.name}";

  @Autowired
  public RubygemsContentFacetImpl(@Qualifier(RubygemsFormat.NAME) final FormatStoreManager formatStoreManager) {
    super(formatStoreManager);
  }

  @Override
  public Optional<FluentAsset> getAsset(final String path) {
    return assets().path(checkNotNull(path)).find();
  }

  @Override
  public TempBlob ingest(final Payload payload) {
    return blobs().ingest(payload, HASHING);
  }

  @Override
  public TempBlob ingest(final InputStream in, @Nullable final String contentType) {
    return blobs().ingest(in, contentType, HASHING);
  }

  @Override
  public FluentAsset putGem(final String path, final GemSpec spec, final TempBlob blob) {
    FluentComponent component = components()
        .name(spec.name())
        .version(spec.version())
        .getOrCreate();
    Map<String, Object> attributes = spec.toAttributes();
    FluentAsset asset = assets()
        .path(path)
        .kind(AssetKind.GEM.name())
        .component(component)
        .attributes(ATTRIBUTES, attributes)
        .blob(blob)
        .save();
    if (!attributes.equals(asset.attributes(ATTRIBUTES).backing())) {
      asset = asset.withAttribute(ATTRIBUTES, attributes);
    }
    return asset;
  }

  @Override
  public FluentAsset putFile(
      final String path,
      final AssetKind kind,
      final TempBlob blob,
      final Map<String, Object> attributes)
  {
    FluentAsset asset = assets()
        .path(path)
        .kind(kind.name())
        .blob(blob)
        .save();
    if (!attributes.isEmpty()) {
      asset.withAttribute(ATTRIBUTES, attributes);
    }
    return asset;
  }

  @Override
  public List<FluentAsset> gems(final String name) {
    return streamOf(components().byFilter(NAME_FILTER, Map.of("name", name))::browse)
        .filter(component -> name.equals(component.name()))
        .flatMap(component -> component.assets().stream())
        .filter(asset -> AssetKind.GEM.name().equals(asset.kind()))
        .toList();
  }

  @Override
  public Stream<FluentAsset> allGems() {
    return streamOf(assets().byKind(AssetKind.GEM.name())::browse);
  }

  @Override
  public boolean delete(final String path) {
    return getAsset(path).map(FluentAsset::delete).orElse(false);
  }

  @Override
  protected WritePolicy writePolicy(final Asset asset) {
    WritePolicy writePolicy = super.writePolicy(asset);
    if (!Objects.equals(AssetKind.GEM.name(), asset.kind())) {
      return WritePolicy.ALLOW;
    }
    return writePolicy;
  }

  @Override
  protected String determineContentType(final Asset asset, final Blob blob, final boolean strictValidation) {
    if (AssetKind.GEM.name().equals(asset.kind())) {
      return super.determineContentType(asset, blob, strictValidation);
    }
    String path = asset.path();
    return path.endsWith(".gz") || path.endsWith(".rz") ? RubygemsResponses.BINARY : TEXT_PLAIN;
  }
}
