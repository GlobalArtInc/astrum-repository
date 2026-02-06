
package org.globalart.astrum.repository.content.maintenance;

import java.util.Set;
import java.util.stream.Collectors;
import java.util.stream.Stream;

import org.globalart.astrum.repository.FacetSupport;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.facet.ContentFacetSupport;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.content.store.ComponentStore;

import com.google.common.collect.ImmutableSet;
import org.springframework.context.annotation.Primary;
import org.springframework.context.annotation.Scope;

import static org.springframework.beans.factory.config.ConfigurableBeanFactory.SCOPE_PROTOTYPE;

/**
 * Default {@link ContentMaintenanceFacet} for formats that don't need additional bookkeeping.
 *
 * @since 3.26
 */
@Scope(SCOPE_PROTOTYPE)
@org.springframework.stereotype.Component
@Primary
public class DefaultMaintenanceFacet
    extends FacetSupport
    implements ContentMaintenanceFacet
{
  @Override
  public Set<String> deleteComponent(final Component component) {
    ImmutableSet.Builder<String> deletedPaths = ImmutableSet.builder();

    FluentComponent componentToDelete = contentFacet().components().with(component);

    componentToDelete.assets().forEach(assetToDelete -> {
      if (assetToDelete.delete()) {
        deletedPaths.add(assetToDelete.path()); // only add paths which were deleted by us
      }
    });

    componentToDelete.delete(); // the component itself has no path

    return deletedPaths.build();
  }

  @Override
  public Set<String> deleteAsset(final Asset asset) {
    ImmutableSet.Builder<String> deletedPaths = ImmutableSet.builder();

    FluentAsset assetToDelete = contentFacet().assets().with(asset);

    if (assetToDelete.delete()) {
      deletedPaths.add(assetToDelete.path());
    }

    return deletedPaths.build();
  }

  protected ContentFacet contentFacet() {
    return facet(ContentFacet.class);
  }

  @Override
  public int deleteComponents(final Stream<FluentComponent> components) {
    ContentFacetSupport contentFacet = (ContentFacetSupport) contentFacet();
    ComponentStore<?> componentStore = contentFacet.stores().componentStore;

    return componentStore.purge(contentFacet.contentRepositoryId(), components.collect(Collectors.toList()));
  }
}
