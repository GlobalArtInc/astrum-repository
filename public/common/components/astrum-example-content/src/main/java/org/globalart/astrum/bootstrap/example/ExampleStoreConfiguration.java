
package org.globalart.astrum.bootstrap.example;

import org.globalart.astrum.content.example.internal.store.ExampleStoresProvider;

import org.springframework.beans.factory.support.BeanDefinitionRegistryPostProcessor;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

/**
 * Configures the content store bindings for an 'example' format.
 */
@Configuration
public class ExampleStoreConfiguration
{
  @Bean
  public static BeanDefinitionRegistryPostProcessor exampleStoresProvider() {
    return new ExampleStoresProvider();
  }
}
