
package org.globalart.astrum.internal.log;

import org.globalart.astrum.security.config.MemorySecurityConfiguration;
import org.globalart.astrum.security.config.SecurityContributor;
import org.globalart.astrum.security.config.SecurityContributorSupport;

import static org.apache.commons.lang3.StringUtils.capitalize;
import org.springframework.stereotype.Component;

/**
 * Logging security configuration.
 *
 * @since 3.0
 */
@Component
public class LoggingSecurityContributor
    extends SecurityContributorSupport
    implements SecurityContributor
{
  private static final String DOMAIN_VALUE = "logging";

  private static final String MARK_DESCRIPTION_BASE = "Mark permission for ";

  public static final String LOGGING_ALL_PRIV_ID = "as-logging-all";

  public static final String LOGGING_READ_PRIV_ID = "as-logging-read";

  public static final String LOGGING_UPDATE_PRIV_ID = "as-logging-update";

  public static final String LOGGING_MARK_PRIV_ID = "as-logging-mark";

  @Override
  public MemorySecurityConfiguration getContribution() {
    MemorySecurityConfiguration configuration = new MemorySecurityConfiguration();

    configuration.addPrivilege(
        createApplicationPrivilege(LOGGING_ALL_PRIV_ID, ALL_DESCRIPTION_BASE + capitalize(DOMAIN_VALUE), DOMAIN_VALUE,
            ACTION_ALL));
    configuration.addPrivilege(
        createApplicationPrivilege(LOGGING_READ_PRIV_ID, READ_DESCRIPTION_BASE + capitalize(DOMAIN_VALUE), DOMAIN_VALUE,
            ACTION_READ));
    configuration.addPrivilege(
        createApplicationPrivilege(LOGGING_UPDATE_PRIV_ID, UPDATE_DESCRIPTION_BASE + capitalize(DOMAIN_VALUE),
            DOMAIN_VALUE, ACTION_UPDATE));
    configuration.addPrivilege(
        createApplicationPrivilege(LOGGING_MARK_PRIV_ID, MARK_DESCRIPTION_BASE + capitalize(DOMAIN_VALUE), DOMAIN_VALUE,
            ACTION_CREATE_ONLY));

    return configuration;
  }
}
