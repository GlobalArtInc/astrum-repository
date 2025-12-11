
package org.globalart.astrum.internal.selector;

import org.globalart.astrum.security.config.MemorySecurityConfiguration;
import org.globalart.astrum.security.config.SecurityConfiguration;
import org.globalart.astrum.security.config.SecurityContributor;
import org.globalart.astrum.security.config.SecurityContributorSupport;
import org.springframework.stereotype.Component;

/**
 * Selectors security configuration.
 *
 * @since 3.0
 */
@Component
public class SelectorsSecurityContributor
    extends SecurityContributorSupport
    implements SecurityContributor
{
  public static final String SELECTORS_DOMAIN = "selectors";

  public static final String SELECTORS_PRIV_ID_PREFIX = "as-selectors";

  @Override
  public SecurityConfiguration getContribution() {
    MemorySecurityConfiguration config = new MemorySecurityConfiguration();

    createCrudAndAllApplicationPrivileges(SELECTORS_PRIV_ID_PREFIX, SELECTORS_DOMAIN).forEach(config::addPrivilege);

    return config;
  }
}
