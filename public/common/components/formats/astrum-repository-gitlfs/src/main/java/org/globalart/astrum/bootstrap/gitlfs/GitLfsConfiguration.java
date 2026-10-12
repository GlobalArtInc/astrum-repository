package org.globalart.astrum.bootstrap.gitlfs;

import org.globalart.astrum.repository.gitlfs.internal.lock.GitLfsLockStoreProvider;
import org.globalart.astrum.repository.gitlfs.internal.store.GitLfsStoresProvider;

import org.springframework.beans.factory.support.BeanDefinitionRegistryPostProcessor;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Configuration
public class GitLfsConfiguration
{
  @Bean
  public static BeanDefinitionRegistryPostProcessor gitLfsStoresProvider() {
    return new GitLfsStoresProvider();
  }

  @Bean
  public static BeanDefinitionRegistryPostProcessor gitLfsLockStoreProvider() {
    return new GitLfsLockStoreProvider();
  }
}
