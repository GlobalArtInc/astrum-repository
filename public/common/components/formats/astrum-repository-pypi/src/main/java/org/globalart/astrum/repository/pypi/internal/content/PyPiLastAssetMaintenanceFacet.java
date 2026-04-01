
package org.globalart.astrum.repository.pypi.internal.content;

import java.util.Set;

import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.maintenance.LastAssetMaintenanceFacet;
import org.globalart.astrum.repository.pypi.internal.AssetKind;
import org.globalart.astrum.repository.pypi.internal.PyPiFormat;
import org.globalart.astrum.repository.pypi.internal.PyPiIndexFacet;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;

@org.springframework.stereotype.Component
@Qualifier(PyPiFormat.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class PyPiLastAssetMaintenanceFacet
    extends LastAssetMaintenanceFacet
{
  @Override
  public Set<String> deleteAsset(final Asset asset) {
    getRepository().optionalFacet(PyPiIndexFacet.class).ifPresent(facet -> {
      if (isPackage(asset)) {
        facet.deleteRootIndex();
        asset.component().ifPresent(component -> facet.deleteIndex(component.name()));
      }
    });
    return super.deleteAsset(asset);
  }

  @Override
  public Set<String> deleteComponent(final Component component) {
    getRepository().optionalFacet(PyPiIndexFacet.class).ifPresent(facet -> {
      facet.deleteRootIndex();
      facet.deleteIndex(component.name());
    });
    return super.deleteComponent(component);
  }

  private static boolean isPackage(final Asset asset) {
    return AssetKind.PACKAGE.name().equals(asset.kind()) || AssetKind.PACKAGE_SIGNATURE.name().equals(asset.kind());
  }
}
