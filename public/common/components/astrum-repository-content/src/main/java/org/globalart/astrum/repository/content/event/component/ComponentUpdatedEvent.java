
package org.globalart.astrum.repository.content.event.component;

import org.globalart.astrum.repository.content.Component;

/**
 * Event sent whenever a {@link Component} is updated.
 *
 * @since 3.26
 */
public class ComponentUpdatedEvent
    extends ComponentEvent
{
  protected ComponentUpdatedEvent(final Component component) {
    super(component);
  }
}
