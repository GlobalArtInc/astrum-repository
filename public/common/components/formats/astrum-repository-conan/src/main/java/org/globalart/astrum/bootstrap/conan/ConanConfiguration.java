package org.globalart.astrum.bootstrap.conan;

import org.globalart.astrum.repository.conan.internal.content.store.ConanStoresProvider;

import org.springframework.beans.factory.support.BeanDefinitionRegistryPostProcessor;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Configuration
public class ConanConfiguration
{
  @Bean
  public static BeanDefinitionRegistryPostProcessor conanStoresProvider() {
    return new ConanStoresProvider();
  }
}
