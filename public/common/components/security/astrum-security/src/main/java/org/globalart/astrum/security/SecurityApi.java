
package org.globalart.astrum.security;

import java.util.List;

import org.globalart.astrum.common.script.ScriptApi;
import org.globalart.astrum.security.anonymous.AnonymousConfiguration;
import org.globalart.astrum.security.authz.NoSuchAuthorizationManagerException;
import org.globalart.astrum.security.role.Role;
import org.globalart.astrum.security.user.NoSuchUserManagerException;
import org.globalart.astrum.security.user.User;
import org.globalart.astrum.security.user.UserNotFoundException;

/**
 * Security provisioning capabilities of the repository manager.
 *
 * @since 3.0
 */
public interface SecurityApi
    extends ScriptApi
{
  @Override
  default String getName() {
    return "security";
  }

  /**
   * Set whether or not to allow anonymous access to the system.
   */
  AnonymousConfiguration setAnonymousAccess(boolean enabled);

  /**
   * Add a new User to the system.
   */
  User addUser(
      String id,
      String firstName,
      String lastName,
      String email,
      boolean active,
      String password,
      List<String> roleIds) throws NoSuchUserManagerException;

  /**
   * Add a new Role to the system.
   */
  Role addRole(
      String id,
      String name,
      String description,
      List<String> privileges,
      List<String> roles) throws NoSuchAuthorizationManagerException;

  /**
   * Set the Roles on a given User.
   */
  User setUserRoles(String userId, List<String> roleIds) throws UserNotFoundException, NoSuchUserManagerException;

}
