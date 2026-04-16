package org.globalart.astrum.bootstrap.conda;

import org.globalart.astrum.repository.conda.internal.store.CondaStoresProvider;

import org.springframework.beans.factory.support.BeanDefinitionRegistryPostProcessor;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Configuration
public class CondaConfiguration
{
  @Bean
  public static BeanDefinitionRegistryPostProcessor condaStoresProvider() {
    return new CondaStoresProvider();
  }
}
