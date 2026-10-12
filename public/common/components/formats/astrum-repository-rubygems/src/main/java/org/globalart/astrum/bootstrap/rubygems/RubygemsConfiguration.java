package org.globalart.astrum.bootstrap.rubygems;

import org.globalart.astrum.repository.rubygems.internal.store.RubygemsStoresProvider;

import org.springframework.beans.factory.support.BeanDefinitionRegistryPostProcessor;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Configuration
public class RubygemsConfiguration
{
  @Bean
  public static BeanDefinitionRegistryPostProcessor rubygemsStoresProvider() {
    return new RubygemsStoresProvider();
  }
}
