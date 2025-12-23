
package org.globalart.astrum.security.authz;

import java.util.Collections;
import java.util.HashSet;
import java.util.List;
import java.util.Set;

import org.globalart.astrum.security.internal.DefaultSecuritySystemTest;
import org.globalart.astrum.security.privilege.NoSuchPrivilegeException;
import org.globalart.astrum.security.privilege.Privilege;
import org.globalart.astrum.security.role.NoSuchRoleException;
import org.globalart.astrum.security.role.Role;

import org.springframework.beans.factory.annotation.Qualifier;

/**
 * @see DefaultSecuritySystemTest
 */
@Qualifier("sourceB")
public class MockAuthorizationManagerB
    extends AbstractReadOnlyAuthorizationManager
{
  @Override
  public String getSource() {
    return "sourceB";
  }

  @Override
  public Set<Role> listRoles() {
    Set<Role> roles = new HashSet<Role>();

    Role role1 = new Role();
    role1.setSource(this.getSource());
    role1.setName("Role 1");
    role1.setRoleId("test-role1");
    role1.addPrivilege("from-role1:read");
    role1.addPrivilege("from-role1:delete");

    Role role2 = new Role();
    role2.setSource(this.getSource());
    role2.setName("Role 2");
    role2.setRoleId("test-role2");
    role2.addPrivilege("from-role2:read");
    role2.addPrivilege("from-role2:delete");

    roles.add(role1);
    roles.add(role2);

    return roles;
  }

  @Override
  public Privilege getPrivilege(String privilegeId) throws NoSuchPrivilegeException {
    return null;
  }

  @Override
  public Privilege getPrivilegeByName(final String privilegeName) throws NoSuchPrivilegeException {
    return null;
  }

  @Override
  public List<Privilege> getPrivileges(final Set<String> privilegeIds) {
    return Collections.emptyList();
  }

  @Override
  public Role getRole(String roleId) throws NoSuchRoleException {
    return null;
  }

  @Override
  public Set<Privilege> listPrivileges() {
    return null;
  }
}
