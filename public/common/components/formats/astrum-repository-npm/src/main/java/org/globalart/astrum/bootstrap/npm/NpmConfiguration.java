package org.globalart.astrum.bootstrap.npm;

import org.globalart.astrum.content.npm.internal.store.NpmStoresProvider;

import org.springframework.beans.factory.support.BeanDefinitionRegistryPostProcessor;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Configuration
public class NpmConfiguration
{
  @Bean
  public static BeanDefinitionRegistryPostProcessor npmStoresProvider() {
    return new NpmStoresProvider();
  }
}
