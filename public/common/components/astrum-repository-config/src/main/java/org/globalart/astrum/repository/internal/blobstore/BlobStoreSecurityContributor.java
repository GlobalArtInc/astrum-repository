
package org.globalart.astrum.repository.internal.blobstore;

import org.globalart.astrum.security.config.MemorySecurityConfiguration;
import org.globalart.astrum.security.config.SecurityConfiguration;
import org.globalart.astrum.security.config.SecurityContributor;
import org.globalart.astrum.security.config.SecurityContributorSupport;
import org.springframework.stereotype.Component;

/**
 * BlobStore security configuration.
 *
 * @since 3.0
 */
@Component
public class BlobStoreSecurityContributor
    extends SecurityContributorSupport
    implements SecurityContributor
{
  public static final String BLOBSTORE_DOMAIN = "blobstores";

  public static final String BLOBSTORE_PRIV_ID_PREFIX = "as-blobstores";

  @Override
  public SecurityConfiguration getContribution() {
    MemorySecurityConfiguration config = new MemorySecurityConfiguration();

    createCrudAndAllApplicationPrivileges(BLOBSTORE_PRIV_ID_PREFIX, BLOBSTORE_DOMAIN).forEach(config::addPrivilege);

    return config;
  }
}
