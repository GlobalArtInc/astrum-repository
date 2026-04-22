package org.globalart.astrum.repository.alpine.internal;

import java.util.Collection;
import java.util.List;
import java.util.Set;
import java.util.stream.Stream;

import org.globalart.astrum.repository.alpine.AlpineFormat;
import org.globalart.astrum.repository.alpine.AssetKind;
import org.globalart.astrum.repository.alpine.internal.hosted.AlpineHostedFacet;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.content.maintenance.LastAssetMaintenanceFacet;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;

@org.springframework.stereotype.Component
@Qualifier(AlpineFormat.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class AlpineMaintenanceFacet
    extends LastAssetMaintenanceFacet
{
  @Override
  public Set<String> deleteComponent(final Component component) {
    Set<String> deleted = super.deleteComponent(component);
    rebuildIndexes(deleted);
    return deleted;
  }

  @Override
  public Set<String> deleteAsset(final Asset asset) {
    Set<String> deleted = super.deleteAsset(asset);
    rebuildIndexes(deleted);
    return deleted;
  }

  @Override
  public int deleteComponents(final Stream<FluentComponent> components) {
    List<FluentComponent> toDelete = components.toList();
    List<String> packagePaths = toDelete.stream()
        .flatMap(component -> component.assets().stream())
        .filter(asset -> AssetKind.PACKAGE.name().equals(asset.kind()))
        .map(Asset::path)
        .toList();
    int deleted = super.deleteComponents(toDelete.stream());
    rebuildIndexes(packagePaths);
    return deleted;
  }

  private void rebuildIndexes(final Collection<String> paths) {
    List<String> packagePaths = paths.stream().filter(AlpinePathUtils::isPackage).toList();
    if (packagePaths.isEmpty()) {
      return;
    }
    getRepository().optionalFacet(AlpineHostedFacet.class)
        .ifPresent(hosted -> hosted.rebuildIndexes(packagePaths));
  }
}
