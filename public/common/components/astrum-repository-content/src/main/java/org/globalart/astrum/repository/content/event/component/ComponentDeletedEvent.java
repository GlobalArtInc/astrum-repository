
package org.globalart.astrum.repository.content.event.component;

import org.globalart.astrum.repository.content.Component;

/**
 * Event sent whenever a {@link Component} is deleted.
 *
 * @since 3.26
 */
public class ComponentDeletedEvent
    extends ComponentEvent
{
  public ComponentDeletedEvent(final Component component) {
    super(component);
  }
}
