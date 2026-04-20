
package org.globalart.astrum.repository.composer.internal;

import java.io.IOException;
import java.io.InputStream;
import java.util.Arrays;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Optional;

import javax.annotation.Nullable;

import org.globalart.astrum.common.hash.HashAlgorithm;
import org.globalart.astrum.repository.cache.CacheInfo;
import org.globalart.astrum.repository.composer.AssetKind;
import org.globalart.astrum.repository.composer.ComposerContentFacet;
import org.globalart.astrum.repository.composer.ComposerFormat;
import org.globalart.astrum.repository.config.WritePolicy;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.facet.ContentFacetSupport;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.content.store.FormatStoreManager;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.payloads.TempBlob;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.hash.HashAlgorithm.MD5;
import static org.globalart.astrum.common.hash.HashAlgorithm.SHA1;
import static org.globalart.astrum.common.hash.HashAlgorithm.SHA256;
import static org.globalart.astrum.repository.composer.internal.ComposerAttributes.P_PROJECT;
import static org.globalart.astrum.repository.composer.internal.ComposerAttributes.P_VENDOR;
import static org.globalart.astrum.repository.composer.internal.ComposerPathUtils.normalizeAssetPath;
import static org.globalart.astrum.repository.composer.internal.recipe.ComposerRecipeSupport.SOURCE_REFERENCE_FIELD_NAME;
import static org.globalart.astrum.repository.composer.internal.recipe.ComposerRecipeSupport.SOURCE_TYPE_FIELD_NAME;
import static org.globalart.astrum.repository.composer.internal.recipe.ComposerRecipeSupport.SOURCE_URL_FIELD_NAME;

@Component
@Qualifier(ComposerFormat.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class ComposerContentFacetImpl
    extends ContentFacetSupport
    implements ComposerContentFacet
{
  public static final List<HashAlgorithm> hashAlgorithms = Arrays.asList(MD5, SHA1, SHA256);

  private final ComposerFormatAttributesExtractor composerFormatAttributesExtractor;

  @Autowired
  public ComposerContentFacetImpl(
      @Qualifier(ComposerFormat.NAME) final FormatStoreManager formatStoreManager,
      final ComposerFormatAttributesExtractor composerFormatAttributesExtractor)
  {
    super(formatStoreManager);
    this.composerFormatAttributesExtractor = checkNotNull(composerFormatAttributesExtractor);
  }

  @Override
  protected WritePolicy writePolicy(final Asset asset) {
    WritePolicy writePolicy = super.writePolicy(asset);
    if (writePolicy == WritePolicy.ALLOW_ONCE && !Objects.equals(AssetKind.ZIPBALL.name(), asset.kind())) {
      return WritePolicy.ALLOW;
    }
    return writePolicy;
  }

  @Override
  public Optional<FluentAsset> getAsset(final String path) {
    return assets().path(normalizeAssetPath(path)).find();
  }

  @Override
  public Optional<Content> get(final String assetPath) {
    return getAsset(assetPath).map(FluentAsset::download);
  }

  @Override
  public Content put(final String path, final Payload payload, final AssetKind assetKind) throws IOException {
    try (TempBlob tempBlob = getTempBlob(payload)) {
      FluentAsset asset;
      if (assetKind == AssetKind.ZIPBALL) {
        asset = findOrCreateContentAsset(path, tempBlob, assetKind, null, null, null);
      }
      else {
        asset = findOrCreateMetadataAsset(path, tempBlob, assetKind);
      }
      return asset.markAsCached(payload).download();
    }
  }

  @Override
  public FluentAsset put(
      final String path,
      final Payload payload,
      final String sourceType,
      final String sourceUrl,
      final String sourceReference) throws IOException
  {
    try (TempBlob tempBlob = getTempBlob(payload)) {
      return findOrCreateContentAsset(path, tempBlob, AssetKind.ZIPBALL, sourceType, sourceUrl, sourceReference);
    }
  }

  @Override
  public TempBlob getTempBlob(final Payload payload) {
    checkNotNull(payload);
    return blobs().ingest(payload, hashAlgorithms);
  }

  @Override
  public TempBlob getTempBlob(final InputStream in, @Nullable final String contentType) {
    checkNotNull(in);
    return blobs().ingest(in, contentType, hashAlgorithms);
  }

  @Override
  public void setCacheInfo(final String path, final Content content, final CacheInfo cacheInfo) {
    Asset asset = content.getAttributes().get(Asset.class);
    if (asset == null) {
      log.debug("Attempting to set cache info for non-existent Composer asset {}", path);
      return;
    }
    assets().with(asset).markAsCached(cacheInfo);
  }

  private FluentAsset findOrCreateMetadataAsset(final String path, final TempBlob tempBlob, final AssetKind assetKind) {
    return assets()
        .path(normalizeAssetPath(path))
        .kind(assetKind.name())
        .blob(tempBlob)
        .save();
  }

  private FluentAsset findOrCreateContentAsset(
      final String path,
      final TempBlob tempBlob,
      final AssetKind assetKind,
      @Nullable final String sourceType,
      @Nullable final String sourceUrl,
      @Nullable final String sourceReference) throws IOException
  {
    String assetPath = normalizeAssetPath(path);
    assets().path(assetPath).find().ifPresent(this::checkAttachAllowed);

    String[] parts = assetPath.split("/");
    String vendor = parts[1];
    String project = parts[2];
    String version = parts[3];

    Map<String, Object> attributes = new LinkedHashMap<>();
    attributes.put(P_VENDOR, vendor);
    attributes.put(P_PROJECT, project);
    if (sourceType != null) {
      attributes.put(SOURCE_TYPE_FIELD_NAME, sourceType);
    }
    if (sourceUrl != null) {
      attributes.put(SOURCE_URL_FIELD_NAME, sourceUrl);
    }
    if (sourceReference != null) {
      attributes.put(SOURCE_REFERENCE_FIELD_NAME, sourceReference);
    }
    attributes.putAll(composerFormatAttributesExtractor.extractFromZip(tempBlob));

    FluentComponent component = components()
        .name(project)
        .namespace(vendor)
        .version(version)
        .normalizedVersion(versionNormalizerService().getNormalizedVersionByFormat(version, repository().getFormat()))
        .getOrCreate()
        .withAttribute(ComposerFormat.NAME, attributes);

    return assets()
        .path(assetPath)
        .kind(assetKind.name())
        .component(component)
        .blob(tempBlob)
        .save();
  }
}
