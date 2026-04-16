package org.globalart.astrum.repository.conda.internal;

import java.util.LinkedHashMap;
import java.util.Map;
import java.util.Optional;
import java.util.Set;

import org.globalart.astrum.repository.conda.AssetKind;
import org.globalart.astrum.repository.conda.CondaFormat;
import org.globalart.astrum.repository.conda.internal.hosted.CondaHostedFacet;
import org.globalart.astrum.repository.conda.util.CondaPathUtils;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.content.maintenance.LastAssetMaintenanceFacet;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;

@org.springframework.stereotype.Component
@Qualifier(CondaFormat.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class CondaMaintenanceFacet
    extends LastAssetMaintenanceFacet
{
  @Override
  public Set<String> deleteAsset(final Asset asset) {
    Optional<Component> component = asset.component();
    if (component.isPresent()) {
      FluentComponent fluentComponent = contentFacet().components().with(component.get());
      if (fluentComponent.assets().size() <= 1) {
        return deleteComponent(fluentComponent);
      }
    }
    Map<String, String> subdirs = subdirsOf(asset, new LinkedHashMap<>());
    Set<String> deleted = super.deleteAsset(asset);
    rebuildRepodata(subdirs);
    return deleted;
  }

  @Override
  public Set<String> deleteComponent(final Component component) {
    Map<String, String> subdirs = new LinkedHashMap<>();
    for (FluentAsset asset : contentFacet().components().with(component).assets()) {
      subdirsOf(asset, subdirs);
    }
    Set<String> deleted = super.deleteComponent(component);
    rebuildRepodata(subdirs);
    return deleted;
  }

  private static Map<String, String> subdirsOf(final Asset asset, final Map<String, String> subdirs) {
    if (AssetKind.isPackage(asset.kind())) {
      CondaPathUtils.parsePackagePath(asset.path())
          .ifPresent(packagePath -> subdirs.put(packagePath.getSubdirPath(), packagePath.getSubdir()));
    }
    return subdirs;
  }

  private void rebuildRepodata(final Map<String, String> subdirs) {
    if (subdirs.isEmpty()) {
      return;
    }
    optionalFacet(CondaHostedFacet.class).ifPresent(hosted -> subdirs.forEach((subdirPath, subdir) -> {
      try {
        hosted.rebuildRepodata(subdirPath, subdir);
      }
      catch (RuntimeException e) {
        log.warn("Failed to rebuild {} of {}", CondaPathUtils.repodataPath(subdirPath), getRepository().getName(), e);
      }
    }));
  }
}
