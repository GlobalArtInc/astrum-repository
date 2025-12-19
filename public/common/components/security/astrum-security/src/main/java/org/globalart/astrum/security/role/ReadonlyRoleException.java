
package org.globalart.astrum.security.role;

/**
 * @since 3.19
 */
public class ReadonlyRoleException
    extends RoleException
{
  private static final String ERROR_TEXT = "Role %s is read only.";

  public ReadonlyRoleException(final String roleId, Throwable cause) {
    super(ERROR_TEXT, roleId, cause);
  }

  public ReadonlyRoleException(final String role) {
    this(role, null);
  }
}
