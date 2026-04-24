package org.globalart.astrum.repository.r.internal;

import java.util.Collection;
import java.util.List;
import java.util.Set;
import java.util.stream.Stream;

import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.content.maintenance.LastAssetMaintenanceFacet;
import org.globalart.astrum.repository.r.AssetKind;
import org.globalart.astrum.repository.r.RFormat;
import org.globalart.astrum.repository.r.internal.hosted.RHostedFacet;
import org.globalart.astrum.repository.r.internal.util.RPathUtils;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;

@org.springframework.stereotype.Component
@Qualifier(RFormat.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class RMaintenanceFacet
    extends LastAssetMaintenanceFacet
{
  @Override
  public Set<String> deleteComponent(final Component component) {
    Set<String> deleted = super.deleteComponent(component);
    rebuildPackages(deleted);
    return deleted;
  }

  @Override
  public Set<String> deleteAsset(final Asset asset) {
    Set<String> deleted = super.deleteAsset(asset);
    rebuildPackages(deleted);
    return deleted;
  }

  @Override
  public int deleteComponents(final Stream<FluentComponent> components) {
    List<FluentComponent> toDelete = components.toList();
    List<String> archivePaths = toDelete.stream()
        .flatMap(component -> component.assets().stream())
        .filter(asset -> AssetKind.ARCHIVE.name().equals(asset.kind()))
        .map(Asset::path)
        .toList();
    int deleted = super.deleteComponents(toDelete.stream());
    rebuildPackages(archivePaths);
    return deleted;
  }

  private void rebuildPackages(final Collection<String> paths) {
    List<String> archivePaths = paths.stream().filter(RPathUtils::isArchive).toList();
    if (archivePaths.isEmpty()) {
      return;
    }
    getRepository().optionalFacet(RHostedFacet.class)
        .ifPresent(hosted -> hosted.rebuildPackages(archivePaths));
  }
}
