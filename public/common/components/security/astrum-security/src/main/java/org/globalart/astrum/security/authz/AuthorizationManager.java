
package org.globalart.astrum.security.authz;

import java.util.List;
import java.util.Set;

import org.globalart.astrum.security.privilege.NoSuchPrivilegeException;
import org.globalart.astrum.security.privilege.Privilege;
import org.globalart.astrum.security.role.Role;

/**
 * Authorization manager.
 */
public interface AuthorizationManager
{
  /**
   * The Id if this AuthorizationManager;
   */
  String getSource();

  /**
   * If this AuthorizationManager is writable.
   */
  boolean supportsWrite();

  /**
   * Returns the all Roles from this AuthorizationManager. NOTE: this call could be slow when coming from an external
   * source (i.e. a database) TODO: Consider removing this method.
   */
  Set<Role> listRoles();

  /**
   * Returns the roles from this AuthorizationManager that meet the query
   * 
   * @param query query to search
   * @return the set of matching roles
   */
  Set<Role> searchRoles(String query);

  /**
   * Returns a Role base on an Id.
   */
  Role getRole(String roleId);

  /**
   * Adds a role to this AuthorizationManager.
   */
  Role addRole(Role role);

  /**
   * Updates a role in this AuthorizationManager.
   */
  Role updateRole(Role role);

  /**
   * Removes a role in this AuthorizationManager.
   */
  void deleteRole(String roleId);

  // Privilege CRUDS

  /**
   * Returns the all Privileges from this AuthorizationManager.
   */
  Set<Privilege> listPrivileges();

  /**
   * Returns a Privilege based on its id.
   */
  Privilege getPrivilege(String privilegeId) throws NoSuchPrivilegeException;

  /**
   * Returns a Privilege based on its name
   * 
   * @param privilegeName the name of the privilege to be queried
   * @return a {@link Privilege} object if present
   * @throws NoSuchPrivilegeException if there is no privilege with such name
   */
  Privilege getPrivilegeByName(String privilegeName) throws NoSuchPrivilegeException;

  /**
   * Returns Privileges base on Ids.
   */
  List<Privilege> getPrivileges(Set<String> privilegeIds);

  /**
   * Adds a Privilege to this AuthorizationManager.
   */
  Privilege addPrivilege(Privilege privilege);

  /**
   * Updates a Privilege in this AuthorizationManager.
   */
  Privilege updatePrivilege(Privilege privilege) throws NoSuchPrivilegeException;

  /**
   * Updates a Privilege by its name in this AuthorizationManager
   * 
   * @param privilege the privilege to be updated
   * @return a {@link Privilege} object if updated successfully
   * @throws NoSuchPrivilegeException if there is no privilege with the name sent on the input parameter
   */
  Privilege updatePrivilegeByName(Privilege privilege) throws NoSuchPrivilegeException;

  /**
   * Removes a Privilege in this AuthorizationManager.
   */
  void deletePrivilege(String privilegeId) throws NoSuchPrivilegeException;

  /**
   * Removes a Privilege in this AuthorizationManager.
   * 
   * @param privilegeName the name of the privilege to be deleted
   * @throws NoSuchPrivilegeException if there is no privilege with such name
   */
  void deletePrivilegeByName(String privilegeName) throws NoSuchPrivilegeException;

  default String getRealmName() {
    return null;
  }
}
