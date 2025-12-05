
package org.globalart.astrum.internal.analytics;

import org.globalart.astrum.security.config.MemorySecurityConfiguration;
import org.globalart.astrum.security.config.SecurityConfiguration;
import org.globalart.astrum.security.config.SecurityContributor;
import org.globalart.astrum.security.config.SecurityContributorSupport;
import org.springframework.stereotype.Component;

/**
 * Analytics security configuration.
 *
 * @since 3.0
 */
@Component
public class AnalyticsSecurityContributor
    extends SecurityContributorSupport
    implements SecurityContributor
{
  public static final String ANALYTICS_DOMAIN = "analytics";

  public static final String ANALYTICS_ALL_PRIV_ID = "as-analytics-all";

  public static final String ANALYTICS_ALL_PRIV_DESCRIPTION = "All permissions for Analytics";

  @Override
  public SecurityConfiguration getContribution() {
    MemorySecurityConfiguration config = new MemorySecurityConfiguration();

    config.addPrivilege(
        createApplicationPrivilege(ANALYTICS_ALL_PRIV_ID, ANALYTICS_ALL_PRIV_DESCRIPTION, ANALYTICS_DOMAIN,
            ACTION_ALL));

    return config;
  }
}
