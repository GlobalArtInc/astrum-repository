package org.globalart.astrum.bootstrap.p2;

import org.globalart.astrum.repository.p2.internal.store.P2StoresProvider;

import org.springframework.beans.factory.support.BeanDefinitionRegistryPostProcessor;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Configuration
public class P2Configuration
{
  @Bean
  public static BeanDefinitionRegistryPostProcessor p2StoresProvider() {
    return new P2StoresProvider();
  }
}
