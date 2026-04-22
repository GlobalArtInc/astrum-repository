package org.globalart.astrum.bootstrap.alpine;

import org.globalart.astrum.repository.alpine.internal.store.AlpineStoresProvider;

import org.springframework.beans.factory.support.BeanDefinitionRegistryPostProcessor;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Configuration
public class AlpineConfiguration
{
  @Bean
  public static BeanDefinitionRegistryPostProcessor alpineStoresProvider() {
    return new AlpineStoresProvider();
  }
}
