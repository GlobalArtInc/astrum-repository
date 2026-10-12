package org.globalart.astrum.repository.rubygems.internal;

import java.util.List;
import java.util.Set;
import java.util.stream.Stream;

import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.content.maintenance.LastAssetMaintenanceFacet;
import org.globalart.astrum.repository.rubygems.RubygemsFormat;
import org.globalart.astrum.repository.rubygems.internal.gem.GemCoordinates;
import org.globalart.astrum.repository.rubygems.internal.hosted.RubygemsHostedFacet;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;

@org.springframework.stereotype.Component
@Qualifier(RubygemsFormat.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class RubygemsMaintenanceFacet
    extends LastAssetMaintenanceFacet
{
  @Override
  public Set<String> deleteComponent(final Component component) {
    Set<String> deleted = super.deleteComponent(component);
    invalidateIndexes(deleted.stream());
    return deleted;
  }

  @Override
  public Set<String> deleteAsset(final Asset asset) {
    Set<String> deleted = super.deleteAsset(asset);
    invalidateIndexes(deleted.stream());
    return deleted;
  }

  @Override
  public int deleteComponents(final Stream<FluentComponent> components) {
    List<FluentComponent> toDelete = components.toList();
    int deleted = super.deleteComponents(toDelete.stream());
    if (deleted > 0) {
      invalidateIndexes(Stream.of(GemCoordinates.GEMS_PREFIX));
    }
    return deleted;
  }

  private void invalidateIndexes(final Stream<String> paths) {
    if (paths.anyMatch(path -> path.startsWith(GemCoordinates.GEMS_PREFIX))) {
      getRepository().optionalFacet(RubygemsHostedFacet.class).ifPresent(RubygemsHostedFacet::invalidateIndexes);
    }
  }
}
