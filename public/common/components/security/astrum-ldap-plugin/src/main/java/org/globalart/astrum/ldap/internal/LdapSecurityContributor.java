package org.globalart.astrum.ldap.internal;

import org.globalart.astrum.security.config.MemorySecurityConfiguration;
import org.globalart.astrum.security.config.SecurityConfiguration;
import org.globalart.astrum.security.config.SecurityContributor;
import org.globalart.astrum.security.config.SecurityContributorSupport;
import org.springframework.stereotype.Component;

/**
 * Privileges for managing LDAP servers: {@code as-ldap-all}, {@code as-ldap-create}, {@code as-ldap-read},
 * {@code as-ldap-update} and {@code as-ldap-delete}.
 */
@Component
public class LdapSecurityContributor
    extends SecurityContributorSupport
    implements SecurityContributor
{
  public static final String DOMAIN = "ldap";

  public static final String PRIVILEGE_PREFIX = "as-ldap";

  @Override
  public SecurityConfiguration getContribution() {
    MemorySecurityConfiguration config = new MemorySecurityConfiguration();
    createCrudAndAllApplicationPrivileges(PRIVILEGE_PREFIX, DOMAIN).forEach(config::addPrivilege);
    return config;
  }
}
