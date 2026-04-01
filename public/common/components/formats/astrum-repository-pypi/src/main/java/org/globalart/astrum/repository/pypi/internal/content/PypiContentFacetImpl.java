
package org.globalart.astrum.repository.pypi.internal.content;

import java.io.InputStream;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.stream.Collectors;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.config.WritePolicy;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.facet.ContentFacetSupport;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.content.fluent.FluentQuery;
import org.globalart.astrum.repository.content.store.FormatStoreManager;
import org.globalart.astrum.repository.pypi.content.PypiContentFacet;
import org.globalart.astrum.repository.pypi.internal.AssetKind;
import org.globalart.astrum.repository.pypi.internal.PyPiFormat;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.payloads.TempBlob;

import com.google.common.collect.ImmutableMap;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.entity.Continuations.streamOf;
import static org.globalart.astrum.repository.config.WritePolicy.ALLOW;
import static org.globalart.astrum.repository.config.WritePolicy.ALLOW_ONCE;
import static org.globalart.astrum.repository.pypi.internal.PyPiStorageUtils.HASH_ALGORITHMS;
import static org.globalart.astrum.repository.pypi.internal.PyPiStorageUtils.getNameAttributes;
import static org.globalart.astrum.repository.pypi.internal.content.PyPiDataUtils.copyFormatAttributes;

@Component
@Qualifier(PyPiFormat.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class PypiContentFacetImpl
    extends ContentFacetSupport
    implements PypiContentFacet
{
  @Autowired
  public PypiContentFacetImpl(@Qualifier(PyPiFormat.NAME) final FormatStoreManager formatStoreManager) {
    super(formatStoreManager);
  }

  @Override
  protected WritePolicy writePolicy(final Asset asset) {
    WritePolicy writePolicy = super.writePolicy(asset);
    if (writePolicy == ALLOW_ONCE && !isPackageKind(asset.kind())) {
      return ALLOW;
    }
    return writePolicy;
  }

  private static boolean isPackageKind(@Nullable final String kind) {
    return AssetKind.PACKAGE.name().equals(kind) || AssetKind.PACKAGE_SIGNATURE.name().equals(kind);
  }

  @Override
  public Optional<FluentAsset> getAsset(final String path) {
    checkNotNull(path);
    return assets().path(path).find();
  }

  @Override
  public boolean delete(final String path) {
    checkNotNull(path);
    return assets().path(path).find().map(FluentAsset::delete).orElse(false);
  }

  @Override
  public FluentAsset saveAsset(
      final String packagePath,
      final FluentComponent component,
      final String assetKind,
      final TempBlob tempBlob)
  {
    checkNotNull(packagePath);
    checkNotNull(component);
    checkNotNull(assetKind);
    checkNotNull(tempBlob);
    return assets()
        .path(packagePath)
        .kind(assetKind)
        .component(component)
        .blob(tempBlob)
        .save();
  }

  @Override
  public FluentAsset saveAsset(final String packagePath, final String assetKind, final TempBlob tempBlob) {
    checkNotNull(packagePath);
    checkNotNull(assetKind);
    checkNotNull(tempBlob);
    return assets()
        .path(packagePath)
        .kind(assetKind)
        .blob(tempBlob)
        .save();
  }

  @Override
  public boolean isComponentExists(final String name) {
    return componentsByName(name).count() > 0;
  }

  @Override
  public List<FluentAsset> assetsByComponentName(final String name) {
    return streamOf(componentsByName(name)::browse)
        .flatMap(component -> component.assets().stream())
        .collect(Collectors.toList());
  }

  @Override
  public FluentComponent findOrCreateComponent(final String name, final String version, final String normalizedName) {
    checkNotNull(name);
    checkNotNull(version);
    checkNotNull(normalizedName);
    FluentComponent component = components().name(normalizedName).version(version).getOrCreate();
    copyFormatAttributes(component, getNameAttributes(name));
    return component;
  }

  @Override
  public TempBlob getTempBlob(final Payload payload) {
    checkNotNull(payload);
    return blobs().ingest(payload, HASH_ALGORITHMS);
  }

  @Override
  public TempBlob getTempBlob(final InputStream content, @Nullable final String contentType) {
    return blobs().ingest(content, contentType, HASH_ALGORITHMS);
  }

  private FluentQuery<FluentComponent> componentsByName(final String name) {
    Map<String, Object> params = ImmutableMap.of("nameParam", name);
    return components().byFilter("name = #{filterParams.nameParam}", params);
  }
}
