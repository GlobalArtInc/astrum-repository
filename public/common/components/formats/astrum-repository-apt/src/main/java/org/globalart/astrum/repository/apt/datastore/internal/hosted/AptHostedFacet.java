
package org.globalart.astrum.repository.apt.datastore.internal.hosted;

import java.io.IOException;

import org.globalart.astrum.repository.Facet.Exposed;
import org.globalart.astrum.repository.FacetSupport;
import org.globalart.astrum.repository.apt.datastore.AptContentFacet;
import org.globalart.astrum.repository.apt.datastore.internal.hosted.metadata.AptHostedMetadataFacet;
import org.globalart.astrum.repository.apt.internal.debian.PackageInfo;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.view.Payload;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

/**
 * Apt hosted facet.
 *
 * @since 3.31
 */
@Component
@Exposed
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class AptHostedFacet
    extends FacetSupport
{
  /**
   * Saves asset to the database and triggers metadata recalculation
   *
   * @param assetPath - Asset path
   * @param payload - Request payload
   * @param packageInfo - Package info
   * @return - Returns Fluent asset after successful save.
   */
  public FluentAsset put(
      final String assetPath,
      final Payload payload,
      final PackageInfo packageInfo) throws IOException
  {
    checkNotNull(assetPath);
    checkNotNull(payload);
    checkNotNull(packageInfo);

    FluentAsset asset = content().put(assetPath, payload, packageInfo);
    metadata().addPackageMetadata(asset);
    return asset;
  }

  private AptContentFacet content() {
    return facet(AptContentFacet.class);
  }

  private AptHostedMetadataFacet metadata() {
    return facet(AptHostedMetadataFacet.class);
  }

  /**
   * Method for triggering Apt metadata recalculation.
   */
  public void rebuildMetadata() throws IOException {
    metadata().rebuildMetadata();
  }
}
