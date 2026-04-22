
package org.globalart.astrum.repository.alpine.internal;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Optional;

import org.globalart.astrum.common.hash.HashAlgorithm;
import org.globalart.astrum.repository.alpine.AlpineContentFacet;
import org.globalart.astrum.repository.alpine.AlpineFormat;
import org.globalart.astrum.repository.alpine.AssetKind;
import org.globalart.astrum.repository.alpine.internal.apk.ApkPackage;
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
import static org.globalart.astrum.repository.view.ContentTypes.APPLICATION_GZIP;

@Component
@Qualifier(AlpineFormat.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class AlpineContentFacetImpl
    extends ContentFacetSupport
    implements AlpineContentFacet
{
  public static final List<HashAlgorithm> HASHING = List.of(SHA1, SHA256);

  private static final String PACKAGES_FILTER = "kind = #{filterParams.kind} AND path LIKE #{filterParams.prefix}";

  @Autowired
  public AlpineContentFacetImpl(@Qualifier(AlpineFormat.NAME) final FormatStoreManager formatStoreManager) {
    super(formatStoreManager);
  }

  @Override
  protected WritePolicy writePolicy(final Asset asset) {
    WritePolicy writePolicy = super.writePolicy(asset);
    if (writePolicy == ALLOW_ONCE && !AssetKind.PACKAGE.name().equals(asset.kind())) {
      return ALLOW;
    }
    return writePolicy;
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
  public Content putPackage(final String path, final TempBlob blob, final ApkPackage apkPackage) {
    FluentComponent component = components()
        .name(apkPackage.name())
        .version(apkPackage.version())
        .getOrCreate();
    return assets()
        .path(path)
        .kind(AssetKind.PACKAGE.name())
        .component(component)
        .blob(blob)
        .save()
        .withAttribute(ATTRIBUTES_KEY, apkPackage.toAttributes())
        .download();
  }

  @Override
  public Content putProxied(final String path, final AssetKind assetKind, final Content content) {
    try (TempBlob blob = blobs().ingest(content, HASHING)) {
      FluentAssetBuilder builder = assets().path(path).kind(assetKind.name()).blob(blob);
      if (assetKind == AssetKind.PACKAGE) {
        AlpinePathUtils.nameAndVersion(AlpinePathUtils.fileName(path)).ifPresent(coordinates ->
            builder.component(components()
                .name(coordinates.name())
                .version(coordinates.version())
                .getOrCreate()));
      }
      return builder.save().markAsCached(content).download();
    }
  }

  @Override
  public Content putIndex(final String path, final byte[] index, final Map<String, Object> attributes) {
    try (TempBlob blob = blobs().ingest(new BytesPayload(index, APPLICATION_GZIP), HASHING)) {
      return assets()
          .path(path)
          .kind(AssetKind.APK_INDEX.name())
          .blob(blob)
          .save()
          .withAttribute(ATTRIBUTES_KEY, attributes)
          .download();
    }
  }

  @Override
  public boolean delete(final String path) {
    return getAsset(path).map(FluentAsset::delete).orElse(false);
  }

  @Override
  public List<FluentAsset> packagesIn(final String directory) {
    String prefix = directory + "/";
    FluentQuery<FluentAsset> query = assets().byFilter(PACKAGES_FILTER,
        Map.of("kind", AssetKind.PACKAGE.name(), "prefix", prefix + "%"));
    List<FluentAsset> packages = new ArrayList<>();
    for (FluentAsset asset : iterableOf(query::browse)) {
      String path = asset.path();
      if (path.startsWith(prefix) && path.indexOf('/', prefix.length()) < 0) {
        packages.add(asset);
      }
    }
    return packages;
  }
}
