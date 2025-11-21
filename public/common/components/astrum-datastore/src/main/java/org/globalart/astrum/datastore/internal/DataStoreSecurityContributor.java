
package org.globalart.astrum.datastore.internal;

import org.globalart.astrum.security.config.MemorySecurityConfiguration;
import org.globalart.astrum.security.config.SecurityConfiguration;
import org.globalart.astrum.security.config.SecurityContributor;
import org.globalart.astrum.security.config.SecurityContributorSupport;
import org.springframework.stereotype.Component;

/**
 * DataStore security configuration.
 *
 * @since 3.19
 */
@Component
public class DataStoreSecurityContributor
    extends SecurityContributorSupport
    implements SecurityContributor
{
  public static final String DATASTORE_DOMAIN = "datastores";

  public static final String DATASTORE_PRIV_ID_PREFIX = "as-datastores";

  @Override
  public SecurityConfiguration getContribution() {
    MemorySecurityConfiguration config = new MemorySecurityConfiguration();

    createCrudAndAllApplicationPrivileges(DATASTORE_PRIV_ID_PREFIX, DATASTORE_DOMAIN).forEach(config::addPrivilege);

    return config;
  }
}
