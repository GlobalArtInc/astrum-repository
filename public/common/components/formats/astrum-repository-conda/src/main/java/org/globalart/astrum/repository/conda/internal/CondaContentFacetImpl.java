
package org.globalart.astrum.repository.conda.internal;

import java.io.InputStream;
import java.util.Comparator;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.stream.Collectors;

import javax.annotation.Nullable;

import org.globalart.astrum.common.hash.HashAlgorithm;
import org.globalart.astrum.repository.conda.AssetKind;
import org.globalart.astrum.repository.conda.CondaContentFacet;
import org.globalart.astrum.repository.conda.CondaFormat;
import org.globalart.astrum.repository.config.WritePolicy;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.facet.ContentFacetSupport;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.fluent.FluentAssetBuilder;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.content.store.AssetDAO;
import org.globalart.astrum.repository.content.store.FormatStoreManager;
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
import static org.globalart.astrum.repository.config.WritePolicy.ALLOW;
import static org.globalart.astrum.repository.conda.util.CondaPathUtils.normalizeAssetPath;

@Component
@Qualifier(CondaFormat.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class CondaContentFacetImpl
    extends ContentFacetSupport
    implements CondaContentFacet
{
  public static final List<HashAlgorithm> HASH_ALGORITHMS =
      ImmutableList.of(HashAlgorithm.SHA1, HashAlgorithm.SHA256, HashAlgorithm.MD5);

  private static final String ASSET_KIND_ATTRIBUTE = "asset_kind";

  @Autowired
  public CondaContentFacetImpl(@Qualifier(CondaFormat.NAME) final FormatStoreManager formatStoreManager) {
    super(formatStoreManager);
  }

  @Override
  protected WritePolicy writePolicy(final Asset asset) {
    if (!AssetKind.isPackage(asset.kind())) {
      return ALLOW;
    }
    return super.writePolicy(asset);
  }

  @Override
  public Optional<FluentAsset> getAsset(final String path) {
    checkNotNull(path);
    return assets().path(normalizeAssetPath(path)).find();
  }

  @Override
  public FluentAsset putMetadata(final String path, final AssetKind assetKind, final TempBlob tempBlob) {
    checkNotNull(path);
    checkNotNull(assetKind);
    checkNotNull(tempBlob);
    return assets()
        .path(normalizeAssetPath(path))
        .kind(assetKind.name())
        .attributes(CondaFormat.NAME, Map.of(ASSET_KIND_ATTRIBUTE, assetKind.name()))
        .blob(tempBlob)
        .save();
  }

  @Override
  public FluentAsset putPackage(
      final String path,
      final AssetKind assetKind,
      final String subdir,
      final String name,
      final String version,
      final TempBlob tempBlob,
      @Nullable final Map<String, Object> index)
  {
    checkNotNull(path);
    checkNotNull(assetKind);
    checkNotNull(tempBlob);
    FluentComponent component = components()
        .name(checkNotNull(name))
        .namespace(checkNotNull(subdir))
        .version(checkNotNull(version))
        .getOrCreate();
    Map<String, Object> attributes = new HashMap<>();
    attributes.put(ASSET_KIND_ATTRIBUTE, assetKind.name());
    if (index != null) {
      attributes.put(INDEX_ATTRIBUTE, index);
    }
    FluentAssetBuilder builder = component.asset(normalizeAssetPath(path))
        .kind(assetKind.name())
        .attributes(CondaFormat.NAME, attributes)
        .blob(tempBlob);
    return builder.save();
  }

  @Override
  public List<FluentAsset> packagesIn(final String subdirPath) {
    String prefix = normalizeAssetPath(subdirPath) + "/";
    String filter = "path LIKE #{" + AssetDAO.FILTER_PARAMS + ".pathPrefix}" +
        " AND kind IN (#{" + AssetDAO.FILTER_PARAMS + ".tarKind}, #{" + AssetDAO.FILTER_PARAMS + ".condaKind})";
    Map<String, Object> params = Map.of(
        "pathPrefix", prefix + "%",
        "tarKind", AssetKind.ARCH_TAR_PACKAGE.name(),
        "condaKind", AssetKind.ARCH_CONDA_PACKAGE.name());
    return streamOf(assets().byFilter(filter, params)::browse)
        .filter(asset -> asset.path().startsWith(prefix) && asset.path().indexOf('/', prefix.length()) < 0)
        .sorted(Comparator.comparing(FluentAsset::path))
        .collect(Collectors.toList());
  }

  @Override
  public boolean delete(final String path) {
    checkNotNull(path);
    return getAsset(path).map(FluentAsset::delete).orElse(false);
  }

  @Override
  public TempBlob getTempBlob(final Payload payload) {
    checkNotNull(payload);
    return blobs().ingest(payload, HASH_ALGORITHMS);
  }

  @Override
  public TempBlob getTempBlob(final InputStream in, @Nullable final String contentType) {
    checkNotNull(in);
    return blobs().ingest(in, contentType, HASH_ALGORITHMS);
  }
}
