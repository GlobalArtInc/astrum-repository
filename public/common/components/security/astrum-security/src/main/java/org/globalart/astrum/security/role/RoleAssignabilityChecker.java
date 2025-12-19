
package org.globalart.astrum.security.role;

import org.globalart.astrum.security.Roles;
import org.globalart.astrum.security.SecurityHelper;

import org.apache.shiro.authz.permission.RolePermissionResolver;
import org.apache.shiro.subject.Subject;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Determines whether the current subject is permitted to assign a given role.
 *
 * <p>
 * A non-admin role is assignable if the caller already holds all of the permissions it contains.
 * The {@code as-admin} role is special-cased: only subjects that already have it may assign it.
 */
@Component
public class RoleAssignabilityChecker
{
  private final RolePermissionResolver rolePermissionResolver;

  private final SecurityHelper securityHelper;

  @Autowired
  public RoleAssignabilityChecker(
      final RolePermissionResolver rolePermissionResolver,
      final SecurityHelper securityHelper)
  {
    this.rolePermissionResolver = checkNotNull(rolePermissionResolver);
    this.securityHelper = checkNotNull(securityHelper);
  }

  public boolean isRoleAssignable(final String roleId) {
    Subject subject = securityHelper.subject();
    if (Roles.ADMIN_ROLE_ID.equals(roleId)) {
      return subject.hasRole(Roles.ADMIN_ROLE_ID);
    }
    return rolePermissionResolver.resolvePermissionsInRole(roleId).stream().allMatch(subject::isPermitted);
  }
}
