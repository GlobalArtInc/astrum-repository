
package org.globalart.astrum.repository.content.search;

import java.util.stream.Stream;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.fluent.FluentComponent;

/**
 * Helper to find components that match a particular UI model.
 *
 * @since 3.26
 */
public interface ComponentFinder
{
  Stream<FluentComponent> findComponentsByModel(
      final Repository repository,
      final String searchComponentId,
      final String namespace,
      final String name,
      final String version);
}
