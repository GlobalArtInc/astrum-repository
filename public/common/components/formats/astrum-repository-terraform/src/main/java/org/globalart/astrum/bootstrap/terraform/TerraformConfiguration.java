package org.globalart.astrum.bootstrap.terraform;

import org.globalart.astrum.repository.terraform.internal.store.TerraformStoresProvider;

import org.springframework.beans.factory.support.BeanDefinitionRegistryPostProcessor;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Configuration
public class TerraformConfiguration
{
  @Bean
  public static BeanDefinitionRegistryPostProcessor terraformStoresProvider() {
    return new TerraformStoresProvider();
  }
}
