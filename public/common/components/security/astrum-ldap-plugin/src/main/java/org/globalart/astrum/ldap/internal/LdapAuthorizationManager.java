package org.globalart.astrum.ldap.internal;

import java.util.LinkedHashSet;
import java.util.List;
import java.util.Set;
import java.util.stream.Collectors;

import org.globalart.astrum.security.authz.AbstractReadOnlyAuthorizationManager;
import org.globalart.astrum.security.privilege.NoSuchPrivilegeException;
import org.globalart.astrum.security.privilege.Privilege;
import org.globalart.astrum.security.role.NoSuchRoleException;
import org.globalart.astrum.security.role.Role;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Offers LDAP groups as external roles. Privileges come from a role in the default source whose id is the group
 * name.
 */
@Component
@Qualifier(LdapUserManager.SOURCE)
public class LdapAuthorizationManager
    extends AbstractReadOnlyAuthorizationManager
{
  static final int ROLE_LIMIT = 1000;

  private final LdapManager ldapManager;

  @Autowired
  public LdapAuthorizationManager(final LdapManager ldapManager) {
    this.ldapManager = checkNotNull(ldapManager);
  }

  @Override
  public String getSource() {
    return LdapUserManager.SOURCE;
  }

  @Override
  public Set<Role> listRoles() {
    return toRoles(ldapManager.searchGroups(null, ROLE_LIMIT));
  }

  @Override
  public Set<Role> searchRoles(final String query) {
    return toRoles(ldapManager.searchGroups(query, ROLE_LIMIT));
  }

  @Override
  public Role getRole(final String roleId) {
    boolean exists;
    try {
      exists = ldapManager.groupExists(roleId);
    }
    catch (LdapServerException e) {
      throw new NoSuchRoleException(roleId, e);
    }
    if (!exists) {
      throw new NoSuchRoleException(roleId);
    }
    return toRole(roleId);
  }

  @Override
  public Set<Privilege> listPrivileges() {
    return Set.of();
  }

  @Override
  public Privilege getPrivilege(final String privilegeId) throws NoSuchPrivilegeException {
    throw new NoSuchPrivilegeException(privilegeId);
  }

  @Override
  public Privilege getPrivilegeByName(final String privilegeName) throws NoSuchPrivilegeException {
    throw new NoSuchPrivilegeException(privilegeName);
  }

  @Override
  public List<Privilege> getPrivileges(final Set<String> privilegeIds) {
    return List.of();
  }

  private static Set<Role> toRoles(final List<String> groups) {
    return groups.stream().map(LdapAuthorizationManager::toRole).collect(Collectors.toCollection(LinkedHashSet::new));
  }

  private static Role toRole(final String group) {
    return new Role(group, group, "LDAP group " + group, LdapUserManager.SOURCE, true, Set.of(), Set.of());
  }
}
