package org.globalart.astrum.bootstrap.cocoapods;

import org.globalart.astrum.repository.cocoapods.internal.store.CocoapodsStoresProvider;

import org.springframework.beans.factory.support.BeanDefinitionRegistryPostProcessor;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Configuration
public class CocoapodsConfiguration
{
  @Bean
  public static BeanDefinitionRegistryPostProcessor cocoapodsStoresProvider() {
    return new CocoapodsStoresProvider();
  }
}
