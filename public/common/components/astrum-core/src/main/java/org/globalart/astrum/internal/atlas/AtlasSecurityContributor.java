
package org.globalart.astrum.internal.atlas;

import org.globalart.astrum.security.config.MemorySecurityConfiguration;
import org.globalart.astrum.security.config.SecurityConfiguration;
import org.globalart.astrum.security.config.SecurityContributor;
import org.globalart.astrum.security.config.SecurityContributorSupport;
import org.springframework.stereotype.Component;

/**
 * Atlas security configuration.
 *
 * @since 3.0
 */
@Component
public class AtlasSecurityContributor
    extends SecurityContributorSupport
    implements SecurityContributor
{
  public static final String ATLAS_DOMAIN = "atlas";

  public static final String ATLAS_ALL_PRIV_ID = "as-atlas-all";

  public static final String ATLAS_ALL_PRIV_DESCRIPTION = "All permissions for Support Tools";

  @Override
  public SecurityConfiguration getContribution() {
    MemorySecurityConfiguration config = new MemorySecurityConfiguration();

    config.addPrivilege(
        createApplicationPrivilege(ATLAS_ALL_PRIV_ID, ATLAS_ALL_PRIV_DESCRIPTION, ATLAS_DOMAIN, ACTION_ALL));

    return config;
  }
}
