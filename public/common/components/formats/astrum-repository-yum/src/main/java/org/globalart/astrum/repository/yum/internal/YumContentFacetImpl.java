package org.globalart.astrum.repository.yum.internal;

import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Optional;

import org.globalart.astrum.blobstore.api.Blob;
import org.globalart.astrum.common.hash.HashAlgorithm;
import org.globalart.astrum.repository.config.WritePolicy;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.facet.ContentFacetSupport;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.fluent.FluentAssetBuilder;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.content.fluent.FluentQuery;
import org.globalart.astrum.repository.content.store.FormatStoreManager;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.payloads.BytesPayload;
import org.globalart.astrum.repository.view.payloads.TempBlob;
import org.globalart.astrum.repository.yum.AssetKind;
import org.globalart.astrum.repository.yum.YumContentFacet;
import org.globalart.astrum.repository.yum.YumFormat;
import org.globalart.astrum.repository.yum.internal.rpm.RpmFileName;
import org.globalart.astrum.repository.yum.internal.rpm.RpmPackage;
import org.globalart.astrum.repository.yum.internal.rpm.RpmParser;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.common.entity.Continuations.iterableOf;
import static org.globalart.astrum.common.hash.HashAlgorithm.MD5;
import static org.globalart.astrum.common.hash.HashAlgorithm.SHA1;
import static org.globalart.astrum.common.hash.HashAlgorithm.SHA256;
import static org.globalart.astrum.common.hash.HashAlgorithm.SHA512;
import static org.globalart.astrum.repository.config.WritePolicy.ALLOW;

@Component
@Qualifier(YumFormat.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class YumContentFacetImpl
    extends ContentFacetSupport
    implements YumContentFacet
{
  public static final List<HashAlgorithm> HASHING = List.of(MD5, SHA1, SHA256, SHA512);

  private static final String UNDER_FILTER = "kind = #{filterParams.kind} AND path LIKE #{filterParams.prefix}";

  @Autowired
  public YumContentFacetImpl(@Qualifier(YumFormat.NAME) final FormatStoreManager formatStoreManager) {
    super(formatStoreManager);
  }

  @Override
  protected WritePolicy writePolicy(final Asset asset) {
    if (isMetadata(asset)) {
      return ALLOW;
    }
    return super.writePolicy(asset);
  }

  @Override
  protected String determineContentType(final Asset asset, final Blob blob, final boolean strictValidation) {
    return super.determineContentType(asset, blob, strictValidation && !isMetadata(asset));
  }

  private static boolean isMetadata(final Asset asset) {
    return AssetKind.REPOMD.name().equals(asset.kind()) || AssetKind.REPODATA.name().equals(asset.kind());
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
  public Content putRpm(final String path, final TempBlob blob, final RpmPackage rpmPackage) {
    return assets()
        .path(path)
        .kind(AssetKind.RPM.name())
        .component(component(rpmPackage))
        .blob(blob)
        .save()
        .withAttribute(ATTRIBUTES_KEY, rpmPackage.toAttributes())
        .download();
  }

  @Override
  public Content putOther(final String path, final TempBlob blob) {
    return assets()
        .path(path)
        .kind(AssetKind.OTHER.name())
        .blob(blob)
        .save()
        .download();
  }

  @Override
  public Content putProxied(final String path, final AssetKind assetKind, final Content content) throws IOException {
    try (TempBlob blob = blobs().ingest(content, HASHING)) {
      FluentAssetBuilder builder = assets().path(path).kind(assetKind.name()).blob(blob);
      RpmPackage rpmPackage = null;
      if (assetKind == AssetKind.RPM) {
        rpmPackage = readQuietly(blob, path);
        if (rpmPackage != null) {
          builder.component(component(rpmPackage));
        }
        else {
          Optional<RpmFileName> fileName = RpmFileName.parse(path.substring(path.lastIndexOf('/') + 1));
          if (fileName.isPresent()) {
            builder.component(components()
                .name(fileName.get().name())
                .version(fileName.get().versionRelease())
                .getOrCreate());
          }
        }
      }
      FluentAsset asset = builder.save().markAsCached(content);
      if (rpmPackage != null) {
        asset = asset.withAttribute(ATTRIBUTES_KEY, rpmPackage.toAttributes());
      }
      return asset.download();
    }
  }

  @Override
  public Content putMetadata(
      final String path,
      final AssetKind assetKind,
      final Path file,
      final String contentType,
      final Map<String, Object> attributes) throws IOException
  {
    try (TempBlob blob = blobs().ingest(file, contentType, HASHING, false)) {
      return saveMetadata(path, assetKind, blob, attributes);
    }
  }

  @Override
  public Content putMetadata(
      final String path,
      final AssetKind assetKind,
      final byte[] bytes,
      final String contentType,
      final Map<String, Object> attributes)
  {
    try (TempBlob blob = blobs().ingest(new BytesPayload(bytes, contentType), HASHING)) {
      return saveMetadata(path, assetKind, blob, attributes);
    }
  }

  private Content saveMetadata(
      final String path,
      final AssetKind assetKind,
      final TempBlob blob,
      final Map<String, Object> attributes)
  {
    FluentAsset asset = assets().path(path).kind(assetKind.name()).blob(blob).save();
    if (!attributes.isEmpty()) {
      asset = asset.withAttribute(ATTRIBUTES_KEY, attributes);
    }
    return asset.download();
  }

  @Override
  public boolean delete(final String path) {
    return getAsset(path).map(FluentAsset::delete).orElse(false);
  }

  @Override
  public Iterable<FluentAsset> assetsOfKind(final AssetKind assetKind) {
    FluentQuery<FluentAsset> query = assets().byKind(assetKind.name());
    return iterableOf(query::browse);
  }

  @Override
  public List<FluentAsset> assetsUnder(final String directory, final AssetKind assetKind) {
    String prefix = directory.endsWith("/") ? directory : directory + "/";
    FluentQuery<FluentAsset> query = assets().byFilter(UNDER_FILTER,
        Map.of("kind", assetKind.name(), "prefix", escapeLike(prefix) + "%"));
    List<FluentAsset> assets = new ArrayList<>();
    for (FluentAsset asset : iterableOf(query::browse)) {
      if (asset.path().startsWith(prefix)) {
        assets.add(asset);
      }
    }
    return assets;
  }

  private FluentComponent component(final RpmPackage rpmPackage) {
    return components()
        .name(rpmPackage.name())
        .version(rpmPackage.componentVersion())
        .getOrCreate();
  }

  private RpmPackage readQuietly(final TempBlob blob, final String path) {
    try (InputStream in = blob.get()) {
      return RpmParser.parse(in);
    }
    catch (IOException | IllegalArgumentException e) {
      log.debug("Unable to read the header of {}: {}", path, e.getMessage());
      return null;
    }
  }

  private static String escapeLike(final String value) {
    return value.replace("\\", "\\\\").replace("%", "\\%").replace("_", "\\_");
  }
}
