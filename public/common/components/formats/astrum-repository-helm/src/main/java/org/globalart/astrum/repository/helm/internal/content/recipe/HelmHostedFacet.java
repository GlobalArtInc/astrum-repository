
package org.globalart.astrum.repository.helm.internal.content.recipe;

import java.io.IOException;

import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.Facet.Exposed;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.payloads.TempBlob;
import org.globalart.astrum.repository.helm.HelmAttributes;
import org.globalart.astrum.repository.helm.internal.AssetKind;

/**
 * Helm Hosted Facet
 *
 * @since 3.28
 */
@Exposed
public interface HelmHostedFacet
    extends Facet
{
  Content get(String path);

  String getPath(final HelmAttributes attributes, final AssetKind assetKind) throws IOException;

  void upload(final String path, final Payload payload, final AssetKind assetKind) throws IOException;

  Content upload(
      final String path,
      final TempBlob tempBlob,
      final HelmAttributes helmAttributes,
      final Payload payload,
      final AssetKind assetKind) throws IOException;

  boolean delete(String path);
}
