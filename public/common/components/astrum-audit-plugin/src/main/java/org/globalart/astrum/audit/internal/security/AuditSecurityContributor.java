
package org.globalart.astrum.audit.internal.security;

import org.globalart.astrum.security.config.MemorySecurityConfiguration;
import org.globalart.astrum.security.config.SecurityContributor;
import org.globalart.astrum.security.config.SecurityContributorSupport;

import org.springframework.stereotype.Component;

import static org.apache.commons.lang3.StringUtils.capitalize;

/**
 * Audit log security configuration.
 */
@Component
public class AuditSecurityContributor
    extends SecurityContributorSupport
    implements SecurityContributor
{
  private static final String DOMAIN_VALUE = "audit";

  public static final String AUDIT_READ_PRIV_ID = "as-audit-read";

  public static final String AUDIT_ALL_PRIV_ID = "as-audit-all";

  @Override
  public MemorySecurityConfiguration getContribution() {
    MemorySecurityConfiguration configuration = new MemorySecurityConfiguration();

    configuration.addPrivilege(
        createApplicationPrivilege(
            AUDIT_READ_PRIV_ID,
            "Read " + capitalize(DOMAIN_VALUE),
            DOMAIN_VALUE,
            ACTION_READ));

    configuration.addPrivilege(
        createApplicationPrivilege(
            AUDIT_ALL_PRIV_ID,
            ALL_DESCRIPTION_BASE + capitalize(DOMAIN_VALUE),
            DOMAIN_VALUE,
            ACTION_ALL));

    return configuration;
  }
}
