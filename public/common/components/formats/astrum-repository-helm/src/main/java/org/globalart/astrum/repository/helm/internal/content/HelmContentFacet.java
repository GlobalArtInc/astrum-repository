
package org.globalart.astrum.repository.helm.internal.content;

import java.io.IOException;
import java.util.Optional;

import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.payloads.TempBlob;
import org.globalart.astrum.repository.helm.HelmAttributes;
import org.globalart.astrum.repository.helm.internal.AssetKind;

/**
 * @since 3.28
 */
@Facet.Exposed
public interface HelmContentFacet
    extends ContentFacet
{
  Iterable<FluentAsset> browseAssets();

  Optional<Content> getAsset(String path);

  Content putIndex(String path, Content content, AssetKind assetKind);

  Content putComponent(String path, Content content, AssetKind assetKind) throws IOException;

  Content putComponent(String path, TempBlob tempBlob, HelmAttributes helmAttrs, Content content, AssetKind assetKind);

  TempBlob getTempBlob(Payload payload);

  boolean delete(String path);
}
