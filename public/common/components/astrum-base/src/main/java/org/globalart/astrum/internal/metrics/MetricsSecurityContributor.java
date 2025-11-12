
package org.globalart.astrum.internal.metrics;

import org.globalart.astrum.security.config.MemorySecurityConfiguration;
import org.globalart.astrum.security.config.SecurityContributor;
import org.globalart.astrum.security.config.SecurityContributorSupport;

import static org.apache.commons.lang3.StringUtils.capitalize;
import org.springframework.stereotype.Component;

/**
 * Metrics security configuration.
 *
 * @since 3.0
 */
@Component
public class MetricsSecurityContributor
    extends SecurityContributorSupport
    implements SecurityContributor
{
  private static final String DOMAIN_VALUE = "metrics";

  public static final String METRICS_ALL_PRIV_ID = "as-metrics-all";

  @Override
  public MemorySecurityConfiguration getContribution() {
    MemorySecurityConfiguration configuration = new MemorySecurityConfiguration();

    configuration.addPrivilege(
        createApplicationPrivilege(METRICS_ALL_PRIV_ID, ALL_DESCRIPTION_BASE + capitalize(DOMAIN_VALUE), DOMAIN_VALUE,
            ACTION_ALL));

    return configuration;
  }
}
