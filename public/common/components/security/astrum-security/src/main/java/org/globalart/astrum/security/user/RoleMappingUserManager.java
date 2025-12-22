
package org.globalart.astrum.security.user;

import java.util.Set;

import org.globalart.astrum.security.role.RoleIdentifier;

/**
 * Extends the UserManager interface to allow a UserManager to add roles to users from other UserManagers.
 *
 * For example, a User might come from a JDBC UserManager, but has additional roles mapped in Astrum.
 */
public interface RoleMappingUserManager
    extends UserManager
{
  /**
   * Returns a list of roles for a user.
   */
  Set<RoleIdentifier> getUsersRoles(String userId, String userSource) throws UserNotFoundException;

  /**
   * Sets a users roles.
   */
  void setUsersRoles(
      String userId,
      String userSource,
      Set<RoleIdentifier> roleIdentifiers) throws UserNotFoundException;
}
