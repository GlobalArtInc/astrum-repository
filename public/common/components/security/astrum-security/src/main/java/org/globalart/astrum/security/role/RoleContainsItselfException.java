
package org.globalart.astrum.security.role;

/**
 * Thrown when a {@link Role} contains its own id in the assigned roles.
 * 
 * @since 3.19
 */
public class RoleContainsItselfException
    extends RoleException
{
  public RoleContainsItselfException(final String roleId, Throwable cause) {
    super("Role %s cannot reference itself.", roleId, cause);
  }

  public RoleContainsItselfException(final String role) {
    this(role, null);
  }
}
