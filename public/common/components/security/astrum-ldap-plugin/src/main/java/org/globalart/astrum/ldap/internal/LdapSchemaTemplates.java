package org.globalart.astrum.ldap.internal;

import java.util.List;

final class LdapSchemaTemplates
{
  private LdapSchemaTemplates() {
  }

  static List<LdapSchemaTemplateXO> all() {
    return List.of(
        LdapSchemaTemplateXO.dynamic("Active Directory", "cn=users", "user", "sAMAccountName"),
        LdapSchemaTemplateXO.posixStatic(),
        LdapSchemaTemplateXO.dynamic("Posix with Dynamic Groups", "ou=people", "posixAccount", "uid"),
        LdapSchemaTemplateXO.dynamic("Generic Ldap Server", null, "inetOrgPerson", "uid"));
  }
}
