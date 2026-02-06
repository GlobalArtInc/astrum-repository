
package org.globalart.astrum.repository.content.maintenance;

import java.util.Set;
import java.util.stream.Stream;

import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.fluent.FluentComponent;

/**
 * @since 3.24
 */
@Facet.Exposed
public interface ContentMaintenanceFacet
    extends Facet
{
  /**
   * Deletes a component from the repository; the format may decide to trigger additional deletes/updates.
   *
   * @return paths(s) of deleted asset(s)
   */
  Set<String> deleteComponent(Component component);

  /**
   * Deletes an asset from the repository; the format may decide to trigger additional deletes/updates.
   *
   * @return path(s) of deleted asset(s)
   */
  Set<String> deleteAsset(Asset asset);

  /**
   * Delete a batch of components.
   *
   * @param components the components to delete
   * @return number of components purged
   * @since 3.29
   */
  int deleteComponents(Stream<FluentComponent> components);
}
