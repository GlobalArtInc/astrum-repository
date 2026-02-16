
package org.globalart.astrum.repository.httpbridge.internal;

import org.globalart.astrum.security.FilterChain;
import org.globalart.astrum.security.JwtFilter;
import org.globalart.astrum.security.anonymous.AnonymousFilter;
import org.globalart.astrum.security.authc.AntiCsrfFilter;
import org.globalart.astrum.security.authc.AstrumAuthenticationFilter;
import org.globalart.astrum.security.authc.apikey.ApiKeyAuthenticationFilter;

import org.springframework.boot.autoconfigure.condition.ConditionalOnProperty;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.core.annotation.Order;

import static org.globalart.astrum.common.app.FeatureFlags.JWT_ENABLED;
import static org.globalart.astrum.common.app.FeatureFlags.SESSION_ENABLED;

@Configuration
public class LegacyHttpBridgeConfiguration
{
  /*
   * Bind after core-servlets but before error servlet
   */
  private static final int PRECEDENCE = -0x50000000;

  private static final String LEGACY_CONTENT_MOUNT_POINT = "/content/**";

  private static final String LEGACY_SERVICE_MOUNT_POINT = "/service/local/**";

  @Order(PRECEDENCE)
  @ConditionalOnProperty(value = SESSION_ENABLED, havingValue = "true", matchIfMissing = true)
  @Bean
  public FilterChain legacyContentHttpBridgeFilterChain() {
    return new FilterChain(LEGACY_CONTENT_MOUNT_POINT,
        AstrumAuthenticationFilter.NAME,
        ApiKeyAuthenticationFilter.NAME,
        AnonymousFilter.NAME,
        AntiCsrfFilter.NAME);
  }

  @Order(PRECEDENCE)
  @ConditionalOnProperty(value = SESSION_ENABLED, havingValue = "true", matchIfMissing = true)
  @Bean
  public FilterChain legacyServiceHttpBridgeFilterChain() {
    return new FilterChain(LEGACY_SERVICE_MOUNT_POINT,
        AstrumAuthenticationFilter.NAME,
        ApiKeyAuthenticationFilter.NAME,
        AnonymousFilter.NAME,
        AntiCsrfFilter.NAME);
  }

  @Order(PRECEDENCE)
  @ConditionalOnProperty(value = JWT_ENABLED, havingValue = "true")
  @Bean
  public FilterChain legacyContentHttpBridgeFilterChainJwt() {
    return new FilterChain(LEGACY_CONTENT_MOUNT_POINT,
        AstrumAuthenticationFilter.NAME,
        JwtFilter.NAME,
        ApiKeyAuthenticationFilter.NAME,
        AnonymousFilter.NAME,
        AntiCsrfFilter.NAME);
  }

  @Order(PRECEDENCE)
  @ConditionalOnProperty(value = JWT_ENABLED, havingValue = "true")
  @Bean
  public FilterChain legacyServiceHttpBridgeFilterChainJwt() {
    return new FilterChain(LEGACY_SERVICE_MOUNT_POINT,
        AstrumAuthenticationFilter.NAME,
        JwtFilter.NAME,
        ApiKeyAuthenticationFilter.NAME,
        AnonymousFilter.NAME,
        AntiCsrfFilter.NAME);
  }
}
