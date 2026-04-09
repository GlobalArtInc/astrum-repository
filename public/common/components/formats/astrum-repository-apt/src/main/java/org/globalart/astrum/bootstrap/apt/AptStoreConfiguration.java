
package org.globalart.astrum.bootstrap.apt;

import org.globalart.astrum.repository.apt.datastore.data.AptKeyValueStoreProvider;
import org.globalart.astrum.repository.apt.datastore.internal.store.AptStoresProvider;

import org.springframework.beans.factory.support.BeanDefinitionRegistryPostProcessor;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Configuration
public class AptStoreConfiguration
{
  @Bean
  public static BeanDefinitionRegistryPostProcessor aptStoresProvider() {
    return new AptStoresProvider();
  }

  @Bean
  public static BeanDefinitionRegistryPostProcessor aptKeyValueStoreProvider() {
    return new AptKeyValueStoreProvider();
  }
}
