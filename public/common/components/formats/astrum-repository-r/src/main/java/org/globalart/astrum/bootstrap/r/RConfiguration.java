package org.globalart.astrum.bootstrap.r;

import org.globalart.astrum.repository.r.internal.store.RStoresProvider;

import org.springframework.beans.factory.support.BeanDefinitionRegistryPostProcessor;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Configuration
public class RConfiguration
{
  @Bean
  public static BeanDefinitionRegistryPostProcessor rStoresProvider() {
    return new RStoresProvider();
  }
}
