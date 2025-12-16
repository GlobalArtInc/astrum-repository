
package org.globalart.astrum.bootstrap.security;

import java.util.List;

import org.globalart.astrum.audit.AuditRecorder;
import org.globalart.astrum.common.app.WebFilterPriority;
import org.globalart.astrum.security.JwtHelper;
import org.globalart.astrum.security.JwtSecurityFilter;
import org.globalart.astrum.security.SecurityFilter;
import org.globalart.astrum.security.authc.AstrumAuthenticationFilter;
import org.globalart.astrum.security.authc.apikey.ApiKeyAuthenticationFilter;
import org.globalart.astrum.security.authc.apikey.ApiKeyExtractor;
import org.globalart.astrum.security.jwt.JwtSessionRevocationService;

import org.apache.shiro.web.filter.mgt.FilterChainResolver;
import org.apache.shiro.web.mgt.WebSecurityManager;
import org.springframework.boot.autoconfigure.condition.ConditionalOnProperty;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.core.annotation.Order;

import static org.globalart.astrum.common.app.FeatureFlags.JWT_ENABLED;
import static org.globalart.astrum.common.app.FeatureFlags.SESSION_ENABLED;

@Configuration
public class SecurityConfiguration
{
  @Order(WebFilterPriority.AUTHENTICATION)
  @ConditionalOnProperty(SESSION_ENABLED)
  @Bean
  public SecurityFilter securityFilter(
      final WebSecurityManager webSecurityManager,
      final FilterChainResolver filterChainResolver)
  {
    return new SecurityFilter(webSecurityManager, filterChainResolver);
  }

  @Order(WebFilterPriority.AUTHENTICATION)
  @ConditionalOnProperty(JWT_ENABLED)
  @Bean
  public JwtSecurityFilter jwtSecurityFilter(
      final WebSecurityManager webSecurityManager,
      final FilterChainResolver filterChainResolver,
      final JwtHelper jwtHelper,
      final JwtSessionRevocationService jwtSessionRevocationService,
      final AuditRecorder auditRecorder)
  {
    return new JwtSecurityFilter(webSecurityManager, filterChainResolver, jwtHelper,
        jwtSessionRevocationService, auditRecorder);
  }

  @Order(WebFilterPriority.AUTHENTICATION)
  @Bean
  public ApiKeyAuthenticationFilter apiKeyAuthenticationFilter(final List<ApiKeyExtractor> apiKeysList) {
    return new ApiKeyAuthenticationFilter(apiKeysList);
  }

  @Order(WebFilterPriority.AUTHENTICATION)
  @Bean
  public AstrumAuthenticationFilter astrumAuthenticationFilter() {
    return new AstrumAuthenticationFilter();
  }
}
