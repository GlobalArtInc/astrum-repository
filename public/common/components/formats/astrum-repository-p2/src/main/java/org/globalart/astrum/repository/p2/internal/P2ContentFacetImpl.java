
package org.globalart.astrum.repository.p2.internal;

import java.io.IOException;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.stream.Collectors;

import javax.annotation.Nullable;

import org.globalart.astrum.common.hash.HashAlgorithm;
import org.globalart.astrum.repository.content.facet.ContentFacetSupport;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.fluent.FluentAssetBuilder;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.content.store.AssetDAO;
import org.globalart.astrum.repository.content.store.FormatStoreManager;
import org.globalart.astrum.repository.p2.P2ContentFacet;
import org.globalart.astrum.repository.p2.internal.metadata.P2Attributes;
import org.globalart.astrum.repository.p2.internal.util.P2TempBlobUtils;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.payloads.TempBlob;

import com.google.common.collect.ImmutableList;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.entity.Continuations.streamOf;
import static org.globalart.astrum.repository.p2.internal.util.P2PathUtils.PLUGIN_NAME;

@Component
@Qualifier(P2Format.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class P2ContentFacetImpl
    extends ContentFacetSupport
    implements P2ContentFacet
{
  private static final List<HashAlgorithm> HASH_ALGORITHMS =
      ImmutableList.of(HashAlgorithm.SHA1, HashAlgorithm.SHA256, HashAlgorithm.MD5);

  private final P2TempBlobUtils p2TempBlobUtils;

  @Autowired
  public P2ContentFacetImpl(
      @Qualifier(P2Format.NAME) final FormatStoreManager formatStoreManager,
      final P2TempBlobUtils p2TempBlobUtils)
  {
    super(formatStoreManager);
    this.p2TempBlobUtils = checkNotNull(p2TempBlobUtils);
  }

  @Override
  public Optional<Content> get(final String path) {
    return assets().path(path).find().map(FluentAsset::download);
  }

  @Override
  public boolean delete(final String path) {
    return assets().path(path).find().map(FluentAsset::delete).orElse(false);
  }

  @Override
  public Content putBinary(final P2Attributes p2Attributes, final Payload payload) {
    try (TempBlob blob = blobs().ingest(payload, HASH_ALGORITHMS)) {
      return putComponent(p2Attributes, blob, payload, AssetKind.BINARY_BUNDLE);
    }
  }

  @Override
  public Content putBundle(final P2Attributes p2Attributes, final Payload payload) throws IOException {
    try (TempBlob blob = blobs().ingest(payload, HASH_ALGORITHMS)) {
      P2Attributes merged = p2TempBlobUtils.mergeAttributesFromTempBlob(blob, p2Attributes);
      return putComponent(merged, blob, payload, AssetKind.BUNDLE);
    }
  }

  @Override
  public Content putMetadata(
      final String path,
      final AssetKind assetKind,
      final Payload payload,
      final Map<String, Object> attributes)
  {
    try (TempBlob blob = blobs().ingest(payload, HASH_ALGORITHMS)) {
      return save(assets().path(path), assetKind, attributes, blob, payload);
    }
  }

  @Override
  public List<FluentAsset> compositeMetadata() {
    String filter = "kind = #{" + AssetDAO.FILTER_PARAMS + ".artifacts} OR kind = #{" +
        AssetDAO.FILTER_PARAMS + ".content}";
    Map<String, Object> params = Map.of(
        "artifacts", AssetKind.COMPOSITE_ARTIFACTS.name(),
        "content", AssetKind.COMPOSITE_CONTENT.name());
    return streamOf(assets().byFilter(filter, params)::browse).collect(Collectors.toList());
  }

  @Override
  public void deleteAll() {
    streamOf(assets()::browse).collect(Collectors.toList()).forEach(FluentAsset::delete);
    streamOf(components()::browse).collect(Collectors.toList()).forEach(FluentComponent::delete);
  }

  private Content putComponent(
      final P2Attributes attributes,
      final TempBlob blob,
      final Payload payload,
      final AssetKind assetKind)
  {
    Map<String, Object> formatAttributes = new HashMap<>();
    if (attributes.getPluginName() != null) {
      formatAttributes.put(PLUGIN_NAME, attributes.getPluginName());
    }

    FluentComponent component = components()
        .name(checkNotNull(attributes.getComponentName()))
        .version(checkNotNull(attributes.getComponentVersion()))
        .kind(assetKind.name())
        .attributes(P2Format.NAME, formatAttributes)
        .getOrCreate();

    return save(component.asset(checkNotNull(attributes.getPath())), assetKind, formatAttributes, blob, payload);
  }

  private static Content save(
      final FluentAssetBuilder builder,
      final AssetKind assetKind,
      @Nullable final Map<String, Object> attributes,
      final TempBlob blob,
      final Payload payload)
  {
    if (attributes != null && !attributes.isEmpty()) {
      builder.attributes(P2Format.NAME, attributes);
    }
    return builder
        .kind(assetKind.name())
        .blob(blob)
        .save()
        .markAsCached(payload)
        .download();
  }
}
