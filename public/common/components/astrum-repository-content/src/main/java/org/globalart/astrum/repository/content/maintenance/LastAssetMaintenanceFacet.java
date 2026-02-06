
package org.globalart.astrum.repository.content.maintenance;

import java.util.Optional;
import java.util.Set;

import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.fluent.FluentComponent;

import org.springframework.context.annotation.Primary;
import org.springframework.context.annotation.Scope;

import static org.springframework.beans.factory.config.ConfigurableBeanFactory.SCOPE_PROTOTYPE;

/**
 * {@link ContentMaintenanceFacet} for formats where components should be deleted along with their last asset.
 *
 * @since 3.26
 */
@Primary
@Scope(SCOPE_PROTOTYPE)
@org.springframework.stereotype.Component
public class LastAssetMaintenanceFacet
    extends DefaultMaintenanceFacet
{
  @Override
  public Set<String> deleteAsset(final Asset asset) {
    Optional<Component> component = asset.component();
    if (component.isPresent()) {
      FluentComponent componentToDelete = contentFacet().components().with(component.get());
      if (componentToDelete.assets().size() <= 1) {
        return deleteComponent(componentToDelete);
      }
    }

    return super.deleteAsset(asset);
  }
}
