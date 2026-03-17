
package org.globalart.astrum.bootstrap.raw;

import org.globalart.astrum.content.raw.internal.store.RawStoresProvider;

import org.springframework.beans.factory.support.BeanDefinitionRegistryPostProcessor;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

/**
 * Configures the content store bindings for the raw format.
 */
@Configuration
public class RawConfiguration
{
  @Bean
  public static BeanDefinitionRegistryPostProcessor rawStoresProvider() {
    return new RawStoresProvider();
  }
}
