package org.globalart.astrum.ldap.internal;

import java.util.Set;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * A user found in an LDAP directory.
 *
 * @param username the value of the user id attribute, as stored in the directory
 * @param dn       the distinguished name of the user entry
 * @param groups   the user's groups, empty unless the server maps groups to roles
 */
public record LdapUser(String username, String dn, String realName, String email, Set<String> groups)
{
  public LdapUser {
    checkNotNull(username);
    groups = groups != null ? Set.copyOf(groups) : Set.of();
  }
}
