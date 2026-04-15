package org.globalart.astrum.bootstrap.golang;

import org.globalart.astrum.repository.golang.internal.content.store.GoStoresProvider;

import org.springframework.beans.factory.support.BeanDefinitionRegistryPostProcessor;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Configuration
public class GolangConfiguration
{
  @Bean
  public static BeanDefinitionRegistryPostProcessor golangStoresProvider() {
    return new GoStoresProvider();
  }
}
