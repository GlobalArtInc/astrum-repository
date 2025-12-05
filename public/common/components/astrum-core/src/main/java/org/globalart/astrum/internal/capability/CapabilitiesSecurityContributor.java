
package org.globalart.astrum.internal.capability;

import org.globalart.astrum.security.config.MemorySecurityConfiguration;
import org.globalart.astrum.security.config.SecurityConfiguration;
import org.globalart.astrum.security.config.SecurityContributor;
import org.globalart.astrum.security.config.SecurityContributorSupport;
import org.springframework.stereotype.Component;

/**
 * Capability security configuration.
 *
 * @since 3.0
 */
@Component
public class CapabilitiesSecurityContributor
    extends SecurityContributorSupport
    implements SecurityContributor
{
  public static final String CAPABILITIES_DOMAIN = "capabilities";

  public static final String CAPABILITIES_PRIV_ID_PREFIX = "as-capabilities";

  @Override
  public SecurityConfiguration getContribution() {
    MemorySecurityConfiguration config = new MemorySecurityConfiguration();

    createCrudAndAllApplicationPrivileges(CAPABILITIES_PRIV_ID_PREFIX, CAPABILITIES_DOMAIN)
        .forEach(config::addPrivilege);

    return config;
  }
}
