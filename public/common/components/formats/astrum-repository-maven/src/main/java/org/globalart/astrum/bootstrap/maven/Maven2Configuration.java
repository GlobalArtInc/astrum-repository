
package org.globalart.astrum.bootstrap.maven;

import org.globalart.astrum.content.maven.store.Maven2StoreProvider;

import org.springframework.beans.factory.support.BeanDefinitionRegistryPostProcessor;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

/**
 * Configures the content store bindings for a maven format.
 */
@Configuration
public class Maven2Configuration
{
  @Bean
  public static BeanDefinitionRegistryPostProcessor maven2Stores() {
    return new Maven2StoreProvider();
  }
}
