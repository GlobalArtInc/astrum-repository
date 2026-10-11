package org.globalart.astrum.accesstoken.internal;

import org.globalart.astrum.security.config.MemorySecurityConfiguration;
import org.globalart.astrum.security.config.SecurityConfiguration;
import org.globalart.astrum.security.config.SecurityContributor;
import org.globalart.astrum.security.config.SecurityContributorSupport;

import org.springframework.stereotype.Component;

/**
 * Privileges to see and revoke other users' access tokens; everyone manages their own without one.
 */
@Component
public class AccessTokenSecurityContributor
    extends SecurityContributorSupport
    implements SecurityContributor
{
  public static final String DOMAIN = "accesstokens";

  public static final String PRIVILEGE_PREFIX = "as-accesstokens";

  @Override
  public SecurityConfiguration getContribution() {
    MemorySecurityConfiguration config = new MemorySecurityConfiguration();
    createCrudAndAllApplicationPrivileges(PRIVILEGE_PREFIX, DOMAIN).forEach(config::addPrivilege);
    return config;
  }
}
