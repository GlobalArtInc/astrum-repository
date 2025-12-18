
package org.globalart.astrum.security.privilege;

/**
 * Emitted when a {@link Privilege} has been deleted.
 *
 * @since 3.1
 */
public class PrivilegeDeletedEvent
    extends PrivilegeEvent
{
  public PrivilegeDeletedEvent(final Privilege privilege) {
    super(privilege);
  }
}
