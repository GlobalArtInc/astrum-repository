
package org.globalart.astrum.security.internal;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.security.config.SecurityConfigurationManager;
import org.globalart.astrum.security.config.SecurityContributor;

import org.springframework.context.event.ContextRefreshedEvent;
import org.springframework.context.event.EventListener;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * Notifies {@link SecurityConfigurationManager} as {@link SecurityContributor}s come and go.
 *
 * @since 3.1
 */
@Component
public class SecurityContributorMediator
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final SecurityConfigurationManagerImpl securityConfigurationManagerImpl;

  @Autowired
  public SecurityContributorMediator(final SecurityConfigurationManagerImpl securityConfigurationManagerImpl) {
    this.securityConfigurationManagerImpl = checkNotNull(securityConfigurationManagerImpl);
  }

  @EventListener
  public void on(final ContextRefreshedEvent event) {
    event.getApplicationContext()
        .getBeansOfType(SecurityContributor.class)
        .values()
        .forEach(securityConfigurationManagerImpl::addContributor);
  }
}
