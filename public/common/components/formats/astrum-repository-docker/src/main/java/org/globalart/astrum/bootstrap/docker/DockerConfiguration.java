package org.globalart.astrum.bootstrap.docker;

import org.globalart.astrum.repository.docker.internal.store.DockerStoresProvider;
import org.globalart.astrum.security.FilterChain;
import org.globalart.astrum.security.JwtFilter;
import org.globalart.astrum.security.anonymous.AnonymousFilter;
import org.globalart.astrum.security.authc.AntiCsrfFilter;
import org.globalart.astrum.security.authc.AstrumAuthenticationFilter;
import org.globalart.astrum.security.authc.apikey.ApiKeyAuthenticationFilter;

import org.springframework.beans.factory.support.BeanDefinitionRegistryPostProcessor;
import org.springframework.boot.autoconfigure.condition.ConditionalOnProperty;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

import static org.globalart.astrum.common.app.FeatureFlags.JWT_ENABLED;
import static org.globalart.astrum.common.app.FeatureFlags.SESSION_ENABLED;

@Configuration
public class DockerConfiguration
{
  public static final String V2_MOUNT_POINT = "/v2/**";

  @Bean
  public static BeanDefinitionRegistryPostProcessor dockerStoresProvider() {
    return new DockerStoresProvider();
  }

  @ConditionalOnProperty(value = SESSION_ENABLED, havingValue = "true", matchIfMissing = true)
  @Bean
  public FilterChain dockerV2FilterChain() {
    return new FilterChain(V2_MOUNT_POINT,
        AstrumAuthenticationFilter.NAME,
        ApiKeyAuthenticationFilter.NAME,
        AnonymousFilter.NAME,
        AntiCsrfFilter.NAME);
  }

  @ConditionalOnProperty(value = JWT_ENABLED, havingValue = "true")
  @Bean
  public FilterChain dockerV2FilterChainJwt() {
    return new FilterChain(V2_MOUNT_POINT,
        AstrumAuthenticationFilter.NAME,
        JwtFilter.NAME,
        ApiKeyAuthenticationFilter.NAME,
        AnonymousFilter.NAME,
        AntiCsrfFilter.NAME);
  }
}
