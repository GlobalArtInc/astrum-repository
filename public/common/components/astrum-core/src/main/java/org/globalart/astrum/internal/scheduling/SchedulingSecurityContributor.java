
package org.globalart.astrum.internal.scheduling;

import org.globalart.astrum.security.config.MemorySecurityConfiguration;
import org.globalart.astrum.security.config.SecurityConfiguration;
import org.globalart.astrum.security.config.SecurityContributor;
import org.globalart.astrum.security.config.SecurityContributorSupport;
import org.springframework.stereotype.Component;

/**
 * Scheduling security configuration.
 *
 * @since 3.0
 */
@Component
public class SchedulingSecurityContributor
    extends SecurityContributorSupport
    implements SecurityContributor
{
  public static final String SCHEDULING_DOMAIN = "tasks";

  public static final String SCHEDULING_PRIV_ID_PREFIX = "as-tasks";

  public static final String SCHEDULING_RUN_PRIV_ID = "as-tasks-run";

  public static final String SCHEDULING_RUN_PRIV_DESCRIPTION = "Run permission for Scheduled Tasks";

  public static final String SCHEDULING_RUN_PRIV_ACTIONS = "start,stop";

  @Override
  public SecurityConfiguration getContribution() {
    MemorySecurityConfiguration config = new MemorySecurityConfiguration();

    createCrudAndAllApplicationPrivileges(SCHEDULING_PRIV_ID_PREFIX, SCHEDULING_DOMAIN).forEach(config::addPrivilege);

    config.addPrivilege(
        createApplicationPrivilege(SCHEDULING_RUN_PRIV_ID, SCHEDULING_RUN_PRIV_DESCRIPTION, SCHEDULING_DOMAIN,
            SCHEDULING_RUN_PRIV_ACTIONS));

    return config;
  }
}
