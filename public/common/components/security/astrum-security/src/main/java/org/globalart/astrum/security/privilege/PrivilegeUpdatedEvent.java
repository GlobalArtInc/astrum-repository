
package org.globalart.astrum.security.privilege;

/**
 * Emitted when a {@link Privilege} has been updated.
 *
 * @since 3.1
 */
public class PrivilegeUpdatedEvent
    extends PrivilegeEvent
{
  public PrivilegeUpdatedEvent(final Privilege privilege) {
    super(privilege);
  }
}
