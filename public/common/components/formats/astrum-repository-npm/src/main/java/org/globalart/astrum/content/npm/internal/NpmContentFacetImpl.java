
package org.globalart.astrum.content.npm.internal;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;
import org.springframework.context.annotation.Scope;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;

import java.io.IOException;
import java.util.List;
import java.util.Map;
import java.util.Optional;

import javax.annotation.Nullable;

import org.globalart.astrum.common.hash.HashAlgorithm;
import org.globalart.astrum.repository.config.WritePolicy;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.facet.ContentFacetSupport;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.fluent.FluentAssetBuilder;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.content.fluent.FluentComponentBuilder;
import org.globalart.astrum.content.npm.NpmContentFacet;
import org.globalart.astrum.repository.content.store.FormatStoreManager;
import org.globalart.astrum.repository.npm.internal.NpmAttributes.AssetKind;
import org.globalart.astrum.repository.npm.internal.NpmFormat;
import org.globalart.astrum.repository.npm.internal.NpmPackageId;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.payloads.TempBlob;

import com.google.common.collect.ImmutableList;
import org.apache.commons.lang3.StringUtils;

import static org.globalart.astrum.common.hash.HashAlgorithm.MD5;
import static org.globalart.astrum.common.hash.HashAlgorithm.SHA1;
import static org.globalart.astrum.repository.config.WritePolicy.ALLOW;
import static org.globalart.astrum.repository.config.WritePolicy.ALLOW_ONCE;
import static org.globalart.astrum.content.npm.NpmContentFacet.metadataPath;
import static org.globalart.astrum.content.npm.NpmContentFacet.tarballPath;

/**
 * @since 3.28
 */
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
@Qualifier(NpmFormat.NAME)
public class NpmContentFacetImpl
    extends ContentFacetSupport
    implements NpmContentFacet
{
  public static final List<HashAlgorithm> HASHING = ImmutableList.of(SHA1, MD5);

  @Autowired
  public NpmContentFacetImpl(@Qualifier(NpmFormat.NAME) final FormatStoreManager formatStoreManager) {
    super(formatStoreManager);
  }

  @Override
  public Optional<Content> get(final NpmPackageId packageId) throws IOException {
    return findAsset(metadataPath(packageId))
        .map(FluentAsset::download);
  }

  @Override
  public FluentAsset put(final NpmPackageId packageId, final Payload content) throws IOException {
    try (TempBlob blob = blobs().ingest(content, HASHING)) {
      String path = metadataPath(packageId);

      save(content, null, AssetKind.PACKAGE_ROOT, null, blob, path);
      return findAsset(path).get();
    }
  }

  @Override
  public boolean delete(final NpmPackageId packageId) throws IOException {
    return maybeDeleteAsset(metadataPath(packageId));
  }

  @Override
  public Optional<Content> get(final NpmPackageId packageId, final String version) throws IOException {
    return findAsset(tarballPath(packageId, version))
        .map(FluentAsset::download);
  }

  @Override
  public FluentAsset put(final NpmPackageId packageId, final TempBlob tempBlob) throws IOException {
    String path = metadataPath(packageId);
    save(null, null, AssetKind.PACKAGE_ROOT, null, tempBlob, path);
    return findAsset(path).get();
  }

  @Override
  public Content put(
      final NpmPackageId packageId,
      final String version,
      final Map<String, Object> npmAttributes,
      final Payload content)
  {
    return putTarball(packageId, version, npmAttributes, content, tarballPath(packageId, version));
  }

  @Override
  public Content put(
      final NpmPackageId packageId,
      final String tarballName,
      final String version,
      final Map<String, Object> npmAttributes,
      final Payload content)
  {
    return putTarball(packageId, version, npmAttributes, content, tarballPath(packageId.id(), tarballName));
  }

  private Content putTarball(
      final NpmPackageId packageId,
      final String version,
      final Map<String, Object> npmAttributes,
      final Payload content,
      final String path)
  {
    try (TempBlob blob = blobs().ingest(content, HASHING)) {
      FluentComponent component = getOrCreateComponent(packageId, version);
      return save(content, component, AssetKind.TARBALL, npmAttributes, blob, path);
    }
  }

  @Override
  public Content put(
      final NpmPackageId packageId,
      final String version,
      final Map<String, Object> npmAttributes,
      final TempBlob blob) throws IOException
  {
    FluentComponent component = getOrCreateComponent(packageId, version);
    String path = tarballPath(packageId, version);

    return save(null, component, AssetKind.TARBALL, npmAttributes, blob, path);
  }

  @Override
  public boolean delete(final NpmPackageId packageId, final String version) throws IOException {
    boolean assetDeleted = maybeDeleteAsset(tarballPath(packageId, version));

    if (assetDeleted) {
      maybeDeleteComponent(packageId, version);
    }

    return assetDeleted;
  }

  private FluentComponent getOrCreateComponent(final NpmPackageId packageId, final String version) {
    FluentComponentBuilder builder = components()
        .name(packageId.name())
        .version(version);

    if (packageId.scope() != null) {
      builder.namespace(packageId.scope());
    }

    return builder.getOrCreate();
  }

  private void maybeDeleteComponent(final NpmPackageId packageId, final String version) {
    FluentComponentBuilder component = components()
        .name(packageId.name())
        .version(version);

    if (packageId.scope() != null) {
      component.namespace(packageId.scope());
    }

    component.find().ifPresent(NpmContentFacetImpl::deleteIfNoAssetsLeft);
  }

  private Content save(
      @Nullable final Payload content,
      @Nullable final FluentComponent component,
      final AssetKind assetKind,
      @Nullable final Map<String, Object> npmAttributes,
      final TempBlob blob,
      final String path)
  {
    FluentAssetBuilder assetBuilder = assets().path(path).kind(assetKind.name());
    if (component != null) {
      assetBuilder = assetBuilder.component(component);
    }

    FluentAsset asset = assetBuilder.blob(blob).save().markAsCached(content);

    if (npmAttributes != null && !npmAttributes.isEmpty()) {
      asset.withAttribute(NpmFormat.NAME, npmAttributes);
    }

    return asset.download();
  }

  private boolean maybeDeleteAsset(final String path) {
    return findAsset(path).map(FluentAsset::delete).orElse(false);
  }

  private Optional<FluentAsset> findAsset(final String path) {
    return assets().path(path).find();
  }

  private static void deleteIfNoAssetsLeft(final FluentComponent component) {
    if (component.assets().isEmpty()) {
      component.delete();
    }
  }

  @Override
  protected WritePolicy writePolicy(final Asset asset) {
    WritePolicy configuredWritePolicy = super.writePolicy(asset);
    if (ALLOW_ONCE == configuredWritePolicy) {
      String assetKind = asset.kind();
      if (StringUtils.equals(AssetKind.PACKAGE_ROOT.name(), assetKind)
          || StringUtils.equals(AssetKind.REPOSITORY_ROOT.name(), assetKind)) {
        return ALLOW;
      }
    }
    return configuredWritePolicy;
  }
}
