
package org.globalart.astrum.internal.wonderland;

import org.globalart.astrum.security.config.MemorySecurityConfiguration;
import org.globalart.astrum.security.config.SecurityContributor;
import org.globalart.astrum.security.config.SecurityContributorSupport;

import static org.apache.commons.lang3.StringUtils.capitalize;
import org.springframework.stereotype.Component;

/**
 * Wonderland security configuration.
 *
 * @since 3.0
 */
@Component
public class WonderlandSecurityContributor
    extends SecurityContributorSupport
    implements SecurityContributor
{
  private static final String DOMAIN_VALUE = "wonderland";

  private static final String AS_WONDERLAND_ALL_PRIV_ID = "as-wonderland-all";

  @Override
  public MemorySecurityConfiguration getContribution() {
    MemorySecurityConfiguration configuration = new MemorySecurityConfiguration();

    configuration.addPrivilege(
        createApplicationPrivilege(AS_WONDERLAND_ALL_PRIV_ID, ALL_DESCRIPTION_BASE + capitalize(DOMAIN_VALUE),
            DOMAIN_VALUE, ACTION_ALL));

    return configuration;
  }
}
