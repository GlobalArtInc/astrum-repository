
package org.globalart.astrum.security.role;

/**
 * @since 3.19
 */
public class DuplicateRoleException
    extends RoleException
{
  private static final String ERROR_TEXT = "Role %s already exists.";

  public DuplicateRoleException(final String roleId, Throwable cause) {
    super(ERROR_TEXT, roleId, cause);
  }

  public DuplicateRoleException(final String role) {
    this(role, null);
  }
}
