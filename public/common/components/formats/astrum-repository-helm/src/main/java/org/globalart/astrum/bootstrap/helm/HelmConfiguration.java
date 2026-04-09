package org.globalart.astrum.bootstrap.helm;

import org.globalart.astrum.repository.helm.internal.content.store.HelmStoresProvider;

import org.springframework.beans.factory.support.BeanDefinitionRegistryPostProcessor;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Configuration
public class HelmConfiguration
{
  @Bean
  public static BeanDefinitionRegistryPostProcessor helmStoresProvider() {
    return new HelmStoresProvider();
  }
}
