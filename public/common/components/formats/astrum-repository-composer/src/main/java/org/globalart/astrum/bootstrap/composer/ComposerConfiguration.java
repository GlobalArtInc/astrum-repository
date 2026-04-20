package org.globalart.astrum.bootstrap.composer;

import org.globalart.astrum.repository.composer.store.ComposerStoresProvider;

import org.springframework.beans.factory.support.BeanDefinitionRegistryPostProcessor;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Configuration
public class ComposerConfiguration
{
  @Bean
  public static BeanDefinitionRegistryPostProcessor composerStoresProvider() {
    return new ComposerStoresProvider();
  }
}
