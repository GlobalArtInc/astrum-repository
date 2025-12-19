
package org.globalart.astrum.security.role;

/**
 * Emitted when a {@link Role} has been created.
 *
 * @since 3.1
 */
public class RoleCreatedEvent
    extends RoleEvent
{
  public RoleCreatedEvent(final Role role) {
    super(role);
  }
}
