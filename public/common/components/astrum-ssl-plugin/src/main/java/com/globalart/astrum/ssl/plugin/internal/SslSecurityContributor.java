
package com.globalart.astrum.ssl.plugin.internal;

import org.globalart.astrum.security.config.MemorySecurityConfiguration;
import org.globalart.astrum.security.config.SecurityConfiguration;
import org.globalart.astrum.security.config.SecurityContributor;
import org.globalart.astrum.security.config.SecurityContributorSupport;
import org.springframework.stereotype.Component;

/**
 * SSL security configuration.
 *
 * @since 3.0
 */
@Component
public class SslSecurityContributor
    extends SecurityContributorSupport
    implements SecurityContributor
{
  public static final String SSL_DOMAIN = "ssl-truststore";

  public static final String SSL_PRIV_ID_PREFIX = "as-ssl-truststore";

  @Override
  public SecurityConfiguration getContribution() {
    MemorySecurityConfiguration config = new MemorySecurityConfiguration();

    createCrudAndAllApplicationPrivileges(SSL_PRIV_ID_PREFIX, SSL_DOMAIN).forEach(config::addPrivilege);

    return config;
  }
}
