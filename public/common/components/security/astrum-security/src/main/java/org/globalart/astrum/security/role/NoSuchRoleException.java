
package org.globalart.astrum.security.role;

/**
 * Thrown when a {@link Role} could not be found.
 */
public class NoSuchRoleException
    extends RoleException
{
  private static final long serialVersionUID = -3551757972830003397L;

  public NoSuchRoleException(final String roleId, Throwable cause) {
    super("Role not found: %s", roleId, cause);
  }

  public NoSuchRoleException(final String role) {
    this(role, null);
  }
}
