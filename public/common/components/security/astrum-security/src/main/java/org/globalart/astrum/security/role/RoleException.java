
package org.globalart.astrum.security.role;

/**
 * @since 3.19
 */
public class RoleException
    extends RuntimeException
{
  private final String roleId;

  public RoleException(final String message, final String roleId) {
    super(String.format(message, roleId));
    this.roleId = roleId;
  }

  public RoleException(final String message, final String roleId, Throwable cause) {
    super(String.format(message, roleId), cause);
    this.roleId = roleId;
  }

  public String getRoleId() {
    return roleId;
  }
}
