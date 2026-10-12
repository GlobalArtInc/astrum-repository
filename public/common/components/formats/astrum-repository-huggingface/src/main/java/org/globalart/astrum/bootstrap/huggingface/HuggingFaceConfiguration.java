package org.globalart.astrum.bootstrap.huggingface;

import org.globalart.astrum.repository.huggingface.internal.store.HuggingFaceStoresProvider;

import org.springframework.beans.factory.support.BeanDefinitionRegistryPostProcessor;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Configuration
public class HuggingFaceConfiguration
{
  @Bean
  public static BeanDefinitionRegistryPostProcessor huggingFaceStoresProvider() {
    return new HuggingFaceStoresProvider();
  }
}
