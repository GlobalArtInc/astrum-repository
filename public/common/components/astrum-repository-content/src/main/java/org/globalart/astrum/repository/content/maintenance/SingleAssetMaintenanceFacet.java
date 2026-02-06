
package org.globalart.astrum.repository.content.maintenance;

import java.util.Optional;
import java.util.Set;

import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.Component;

import org.springframework.context.annotation.Primary;
import org.springframework.context.annotation.Scope;

import static org.springframework.beans.factory.config.ConfigurableBeanFactory.SCOPE_PROTOTYPE;

/**
 * {@link ContentMaintenanceFacet} for formats where components have a one-to-one association with assets.
 *
 * @since 3.26
 */
@Primary
@Scope(SCOPE_PROTOTYPE)
@org.springframework.stereotype.Component
public class SingleAssetMaintenanceFacet
    extends DefaultMaintenanceFacet
{
  @Override
  public Set<String> deleteAsset(final Asset asset) {
    Optional<Component> component = asset.component();
    if (component.isPresent()) {
      return super.deleteComponent(component.get());
    }

    return super.deleteAsset(asset);
  }
}
