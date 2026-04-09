
package org.globalart.astrum.repository.helm.internal.content.recipe;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;
import org.springframework.context.annotation.Scope;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;

import java.io.IOException;
import java.util.Objects;
import java.util.Optional;


import org.globalart.astrum.common.hash.HashAlgorithm;
import org.globalart.astrum.repository.config.WritePolicy;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.facet.ContentFacetSupport;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.store.FormatStoreManager;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.payloads.TempBlob;
import org.globalart.astrum.repository.helm.HelmAttributes;
import org.globalart.astrum.repository.helm.internal.AssetKind;
import org.globalart.astrum.repository.helm.internal.HelmFormat;
import org.globalart.astrum.repository.helm.internal.content.HelmContentFacet;
import org.globalart.astrum.repository.helm.internal.metadata.IndexYamlAbsoluteUrlRewriter;
import org.globalart.astrum.repository.helm.internal.util.HelmAttributeParser;

import com.google.common.collect.ImmutableList;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.hash.HashAlgorithm.MD5;
import static org.globalart.astrum.common.hash.HashAlgorithm.SHA1;
import static org.globalart.astrum.repository.config.WritePolicy.ALLOW_ONCE;
import static org.globalart.astrum.repository.helm.internal.AssetKind.HELM_PACKAGE;

/**
 * @since 3.28
 */
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
@Qualifier(HelmFormat.NAME)
public class HelmContentFacetImpl
    extends ContentFacetSupport
    implements HelmContentFacet
{
  private static final Iterable<HashAlgorithm> HASHING = ImmutableList.of(MD5, SHA1);

  private final HelmAttributeParser helmAttributeParser;

  private final IndexYamlAbsoluteUrlRewriter indexYamlRewriter;

  @Autowired
  public HelmContentFacetImpl(
      @Qualifier(HelmFormat.NAME) final FormatStoreManager formatStoreManager,
      final HelmAttributeParser helmAttributeParser,
      final IndexYamlAbsoluteUrlRewriter indexYamlRewriter)
  {
    super(formatStoreManager);
    this.helmAttributeParser = checkNotNull(helmAttributeParser);
    this.indexYamlRewriter = checkNotNull(indexYamlRewriter);
  }

  @Override
  protected WritePolicy writePolicy(final Asset asset) {
    WritePolicy writePolicy = super.writePolicy(asset);
    if (writePolicy == ALLOW_ONCE && !Objects.equals(HELM_PACKAGE.name(), asset.kind())) {
      writePolicy = WritePolicy.ALLOW;
    }
    return writePolicy;
  }

  @Override
  public Iterable<FluentAsset> browseAssets() {
    return assets().browse(Integer.MAX_VALUE, null);
  }

  @Override
  public Optional<Content> getAsset(final String path) {
    return assets().path(path).find().map(FluentAsset::download);
  }

  @Override
  public Content putIndex(final String path, final Content content, final AssetKind assetKind)
  {
    // save original metadata and return modified
    try (TempBlob blob = blobs().ingest(content, HASHING)) {
        assets()
            .path(path)
            .kind(assetKind.name())
            .blob(blob)
            .save()
            .markAsCached(content);
      return indexYamlRewriter.removeUrlsFromIndexYaml(blob, content.getAttributes());
    }
  }

  @Override
  public TempBlob getTempBlob(final Payload payload) {
    return blobs().ingest(payload, HASHING);
  }

  @Override
  public Content putComponent(final String path, final Content content, final AssetKind assetKind) throws IOException
  {
    try (TempBlob blob = blobs().ingest(content, HASHING)) {
      HelmAttributes helmAttributes = helmAttributeParser.getAttributes(assetKind, blob.get());

      return assets()
          .path(path)
          .kind(assetKind.name())
          .component(components()
              .name(helmAttributes.getName())
              .version(helmAttributes.getVersion())
              .getOrCreate())
          .blob(blob)
          .save()
          .markAsCached(content)
          .withAttribute(HelmFormat.NAME, helmAttributes)
          .download();
    }
  }

  @Override
  public Content putComponent(
      final String path,
      final TempBlob tempBlob,
      final HelmAttributes helmAttributes,
      final Content content,
      final AssetKind assetKind)
  {
    return assets()
        .path(path)
        .kind(assetKind.name())
        .component(components()
            .name(helmAttributes.getName())
            .version(helmAttributes.getVersion())
            .getOrCreate())
        .blob(tempBlob)
        .save()
        .markAsCached(content)
        .withAttribute(HelmFormat.NAME, helmAttributes)
        .download();
  }

  @Override
  public boolean delete(final String path) {
    return assets().path(path).find().map(FluentAsset::delete).orElse(false);
  }
}
