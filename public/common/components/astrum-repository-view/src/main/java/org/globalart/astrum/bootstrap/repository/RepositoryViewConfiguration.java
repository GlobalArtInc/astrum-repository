
package org.globalart.astrum.bootstrap.repository;

import java.util.Collections;
import java.util.List;

import org.globalart.astrum.repository.view.handlers.ContributedHandler;

import org.springframework.boot.autoconfigure.condition.ConditionalOnMissingBean;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

/**
 * This configuration is essential for the (OSS) version of Astrum.
 * It provides a default empty list for {@code ContributedHandler} beans,
 * preventing a {@code NoSuchBeanDefinitionException} at startup.
 *
 * <p>
 * Professional (PRO) and Community (CE) editions of Astrum include actual
 * implementations of {@code ContributedHandler}, so this bean isn't created
 * for those versions as it's not needed.
 */
@Configuration
public class RepositoryViewConfiguration
{
  @Bean
  @ConditionalOnMissingBean(ContributedHandler.class)
  public List<ContributedHandler> contributedHandlers() {
    return Collections.emptyList();
  }
}
