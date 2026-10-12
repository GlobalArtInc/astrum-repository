package org.globalart.astrum.bootstrap.swift;

import org.globalart.astrum.repository.swift.internal.store.SwiftStoresProvider;

import org.springframework.beans.factory.support.BeanDefinitionRegistryPostProcessor;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Configuration
public class SwiftConfiguration
{
  @Bean
  public static BeanDefinitionRegistryPostProcessor swiftStoresProvider() {
    return new SwiftStoresProvider();
  }
}
