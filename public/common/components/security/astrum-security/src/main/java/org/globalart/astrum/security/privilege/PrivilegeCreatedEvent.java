
package org.globalart.astrum.security.privilege;

/**
 * Emitted when a {@link Privilege} has been created.
 *
 * @since 3.1
 */
public class PrivilegeCreatedEvent
    extends PrivilegeEvent
{
  public PrivilegeCreatedEvent(final Privilege privilege) {
    super(privilege);
  }
}
