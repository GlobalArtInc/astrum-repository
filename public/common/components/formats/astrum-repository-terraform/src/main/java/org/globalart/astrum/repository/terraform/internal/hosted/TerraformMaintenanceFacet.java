package org.globalart.astrum.repository.terraform.internal.hosted;

import java.util.Collection;
import java.util.List;
import java.util.Set;
import java.util.stream.Stream;

import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.content.maintenance.LastAssetMaintenanceFacet;
import org.globalart.astrum.repository.terraform.AssetKind;
import org.globalart.astrum.repository.terraform.TerraformFormat;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;

/**
 * Deletes like the default maintenance and then rewrites the checksums the repository generated for the provider
 * versions that lost a package.
 */
@org.springframework.stereotype.Component
@Qualifier(TerraformFormat.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class TerraformMaintenanceFacet
    extends LastAssetMaintenanceFacet
{
  @Override
  public Set<String> deleteComponent(final Component component) {
    Set<String> deleted = super.deleteComponent(component);
    refresh(deleted);
    return deleted;
  }

  @Override
  public Set<String> deleteAsset(final Asset asset) {
    Set<String> deleted = super.deleteAsset(asset);
    if (AssetKind.PROVIDER_ARCHIVE.name().equals(asset.kind())) {
      refresh(deleted);
    }
    return deleted;
  }

  @Override
  public int deleteComponents(final Stream<FluentComponent> components) {
    List<FluentComponent> toDelete = components.toList();
    List<String> packagePaths = toDelete.stream()
        .flatMap(component -> component.assets().stream())
        .filter(asset -> AssetKind.PROVIDER_ARCHIVE.name().equals(asset.kind()))
        .map(Asset::path)
        .toList();
    int deleted = super.deleteComponents(toDelete.stream());
    refresh(packagePaths);
    return deleted;
  }

  private void refresh(final Collection<String> paths) {
    if (paths.isEmpty()) {
      return;
    }
    getRepository().optionalFacet(TerraformHostedFacet.class).ifPresent(hosted -> hosted.refreshChecksums(paths));
  }
}
