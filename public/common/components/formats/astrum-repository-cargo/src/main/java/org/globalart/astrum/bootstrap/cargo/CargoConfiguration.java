package org.globalart.astrum.bootstrap.cargo;

import org.globalart.astrum.repository.cargo.internal.store.CargoStoresProvider;

import org.springframework.beans.factory.support.BeanDefinitionRegistryPostProcessor;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Configuration
public class CargoConfiguration
{
  @Bean
  public static BeanDefinitionRegistryPostProcessor cargoStoresProvider() {
    return new CargoStoresProvider();
  }
}
