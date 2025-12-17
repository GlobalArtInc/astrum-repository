
package org.globalart.astrum.security.authz;

import java.util.Collection;

import org.apache.shiro.authz.Permission;
import org.apache.shiro.authz.permission.RolePermissionResolver;

/**
 * A {@link RolePermissionResolver} that can resolve an entire set of role IDs in one batch, instead of one call per
 * role. Lets callers (notably {@link PermissionCachingAuthorizingRealm#getPermissions}) collapse O(roles) resolver
 * round-trips into O(depth) batch calls without depending on a concrete resolver implementation (ASTRUM-52583).
 */
public interface BatchRolePermissionResolver
    extends RolePermissionResolver
{
  /**
   * Resolves the union of permissions granted by the given role IDs (transitively through nested roles).
   *
   * @param roleIds the role IDs to expand; may be empty
   * @return the resolved permissions; never {@code null}
   */
  Collection<Permission> resolvePermissionsForRoles(Collection<String> roleIds);
}
