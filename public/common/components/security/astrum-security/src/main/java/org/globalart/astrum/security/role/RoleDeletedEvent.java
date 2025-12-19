
package org.globalart.astrum.security.role;

/**
 * Emitted when a {@link Role} has been deleted.
 *
 * @since 3.1
 */
public class RoleDeletedEvent
    extends RoleEvent
{
  public RoleDeletedEvent(final Role role) {
    super(role);
  }
}
