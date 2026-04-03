package org.globalart.astrum.bootstrap.nuget;

import org.globalart.astrum.repository.nuget.internal.store.NugetStoresProvider;

import org.springframework.beans.factory.support.BeanDefinitionRegistryPostProcessor;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Configuration
public class NugetConfiguration
{
  @Bean
  public static BeanDefinitionRegistryPostProcessor nugetStoresProvider() {
    return new NugetStoresProvider();
  }
}
