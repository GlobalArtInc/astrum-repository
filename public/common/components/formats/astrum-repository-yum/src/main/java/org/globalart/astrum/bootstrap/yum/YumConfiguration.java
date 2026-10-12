package org.globalart.astrum.bootstrap.yum;

import org.globalart.astrum.repository.yum.internal.store.YumStoresProvider;

import org.springframework.beans.factory.support.BeanDefinitionRegistryPostProcessor;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Configuration
public class YumConfiguration
{
  @Bean
  public static BeanDefinitionRegistryPostProcessor yumStoresProvider() {
    return new YumStoresProvider();
  }
}
