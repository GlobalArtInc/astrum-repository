
package org.globalart.astrum.repository.content.event.component;

import org.globalart.astrum.repository.content.Component;

/**
 * Event sent whenever a {@link Component}'s kind is updated.
 *
 * @since 3.26
 */
public class ComponentKindEvent
    extends ComponentUpdatedEvent
{
  public ComponentKindEvent(final Component component) {
    super(component);
  }
}
