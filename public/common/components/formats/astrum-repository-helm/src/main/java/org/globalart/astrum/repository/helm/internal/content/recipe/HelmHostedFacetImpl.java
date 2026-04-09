
package org.globalart.astrum.repository.helm.internal.content.recipe;

import org.springframework.stereotype.Component;
import org.springframework.context.annotation.Scope;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;

import java.io.IOException;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.FacetSupport;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.payloads.TempBlob;
import org.globalart.astrum.repository.helm.HelmAttributes;
import org.globalart.astrum.repository.helm.internal.AssetKind;
import org.globalart.astrum.repository.helm.internal.content.HelmContentFacet;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.helm.internal.AssetKind.HELM_PACKAGE;
import static org.globalart.astrum.repository.helm.internal.AssetKind.HELM_PROVENANCE;

/**
 * {@link HelmHostedFacet implementation}
 *
 * @since 3.28
 */
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class HelmHostedFacetImpl
    extends FacetSupport
    implements HelmHostedFacet
{
  private HelmContentFacet helmContentFacet;

  @Override
  protected void doInit(final Configuration configuration) throws Exception {
    super.doInit(configuration);
    helmContentFacet = facet(HelmContentFacet.class);
  }

  @Nullable
  @Override
  public Content get(final String path) {
    checkNotNull(path);
    return helmContentFacet.getAsset(path).orElse(null);
  }

  @Override
  public String getPath(final HelmAttributes attributes, final AssetKind assetKind)
  {
    if (assetKind != HELM_PACKAGE && assetKind != HELM_PROVENANCE) {
      throw new IllegalArgumentException("Unsupported assetKind: " + assetKind);
    }

    String extension = assetKind.getExtension();
    String name = attributes.getName();
    String version = attributes.getVersion();

    return String.format("/%s-%s%s", name, version, extension);
  }

  @Override
  public void upload(
      final String path,
      final Payload payload,
      final AssetKind assetKind) throws IOException
  {
    checkNotNull(path);
    if (assetKind != HELM_PACKAGE && assetKind != HELM_PROVENANCE) {
      throw new IllegalArgumentException("Unsupported assetKind: " + assetKind);
    }
    helmContentFacet.putComponent(path, new Content(payload), assetKind);
  }

  @Override
  public Content upload(
      final String path,
      final TempBlob tempBlob,
      final HelmAttributes helmAttributes,
      final Payload payload,
      final AssetKind assetKind)
  {
    checkNotNull(path);
    if (assetKind != HELM_PACKAGE && assetKind != HELM_PROVENANCE) {
      throw new IllegalArgumentException("Unsupported assetKind: " + assetKind);
    }
    return helmContentFacet.putComponent(path, tempBlob, helmAttributes, new Content(payload), assetKind);
  }

  @Override
  public boolean delete(final String path) {
    checkNotNull(path);
    return helmContentFacet.delete(path);
  }
}
