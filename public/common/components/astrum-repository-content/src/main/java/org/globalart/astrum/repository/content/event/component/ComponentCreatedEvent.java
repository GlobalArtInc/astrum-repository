
package org.globalart.astrum.repository.content.event.component;

import org.globalart.astrum.repository.content.Component;

/**
 * Event sent whenever a {@link Component} is created.
 *
 * @since 3.26
 */
public class ComponentCreatedEvent
    extends ComponentEvent
{
  public ComponentCreatedEvent(final Component component) {
    super(component);
  }
}
