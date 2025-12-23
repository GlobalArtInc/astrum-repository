
package org.globalart.astrum.security.authz;

import java.util.Collections;
import java.util.HashSet;
import java.util.List;
import java.util.Set;

import org.globalart.astrum.security.privilege.NoSuchPrivilegeException;
import org.globalart.astrum.security.privilege.Privilege;
import org.globalart.astrum.security.role.NoSuchRoleException;
import org.globalart.astrum.security.role.Role;

// FIXME: Appears unused

public class MockAuthorizationManager
    extends AbstractReadOnlyAuthorizationManager
{
  @Override
  public String getSource() {
    return "Mock";
  }

  @Override
  public Set<Role> listRoles() {
    Set<Role> roles = new HashSet<Role>();

    roles.add(new Role("mockrole1", "MockRole1", "Mock Role1", "Mock", true, null, null));
    roles.add(new Role("mockrole2", "MockRole2", "Mock Role2", "Mock", true, null, null));
    roles.add(new Role("mockrole3", "MockRole3", "Mock Role3", "Mock", true, null, null));

    return roles;
  }

  @Override
  public Role getRole(String roleId) throws NoSuchRoleException {
    for (Role role : this.listRoles()) {
      if (roleId.equals(role.getRoleId())) {
        return role;
      }
    }
    throw new NoSuchRoleException(roleId);
  }

  @Override
  public Set<Privilege> listPrivileges() {
    return new HashSet<Privilege>();
  }

  @Override
  public Privilege getPrivilege(String privilegeId) throws NoSuchPrivilegeException {
    throw new NoSuchPrivilegeException(privilegeId);
  }

  @Override
  public Privilege getPrivilegeByName(final String privilegeName) throws NoSuchPrivilegeException {
    throw new NoSuchPrivilegeException(privilegeName);
  }

  @Override
  public List<Privilege> getPrivileges(final Set<String> privilegeIds) {
    return Collections.emptyList();
  }
}
