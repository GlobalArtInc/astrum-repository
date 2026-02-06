
package org.globalart.astrum.repository.content.rest;

import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.rest.api.ComponentXO;

/**
 * Extension point for the {@code ComponentsResource} class
 */
public interface ComponentsResourceExtension
{
  /**
   * Update the {@link ComponentXO} with the data in the {@link FluentComponent}
   */
  ComponentXO updateComponentXO(ComponentXO componentXO, FluentComponent component);
}
