
package org.globalart.astrum.repository.apt.datastore.internal.hosted;

import java.util.Set;

import org.globalart.astrum.repository.apt.AptFormat;
import org.globalart.astrum.repository.apt.datastore.internal.metadata.AptMetadataFacetSupport;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.maintenance.LastAssetMaintenanceFacet;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;

/**
 * Apt maintenance facet
 *
 * @since 3.31
 */
@Qualifier(AptFormat.NAME)
@org.springframework.stereotype.Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class AptLastAssetMaintenanceFacet
    extends LastAssetMaintenanceFacet
{
  @Override
  public Set<String> deleteAsset(final Asset asset) {
    // Remove metadata BEFORE deleting the asset so we can still access it
    removeAssetMetadata(asset);

    return super.deleteAsset(asset);
  }

  @Override
  public Set<String> deleteComponent(final Component component) {
    // Remove metadata for all assets in this component before deletion
    contentFacet().components().with(component).assets().forEach(this::removeAssetMetadata);

    return super.deleteComponent(component);
  }

  private void removeAssetMetadata(final Asset asset) {
    FluentAsset fluentAsset = contentFacet().assets().with(asset);
    metadata().removePackageMetadata(fluentAsset);
  }

  private AptMetadataFacetSupport metadata() {
    return facet(AptMetadataFacetSupport.class);
  }
}
