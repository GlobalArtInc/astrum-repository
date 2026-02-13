
package org.globalart.astrum.repository.httpbridge;

import org.globalart.astrum.repository.httpbridge.internal.ViewServlet;
import org.globalart.astrum.security.FilterChain;
import org.globalart.astrum.security.JwtFilter;
import org.globalart.astrum.security.anonymous.AnonymousFilter;
import org.globalart.astrum.security.authc.AntiCsrfFilter;
import org.globalart.astrum.security.authc.AstrumAuthenticationFilter;
import org.globalart.astrum.security.authc.apikey.ApiKeyAuthenticationFilter;

import org.springframework.boot.autoconfigure.condition.ConditionalOnProperty;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

import static org.globalart.astrum.common.app.FeatureFlags.JWT_ENABLED;
import static org.globalart.astrum.common.app.FeatureFlags.SESSION_ENABLED;

@Configuration
public class HttpBridgeConfiguration
{
  public static final String MOUNT_POINT = ViewServlet.MOUNT_POINT + "*";

  @ConditionalOnProperty(value = SESSION_ENABLED, havingValue = "true", matchIfMissing = true)
  @Bean
  public FilterChain httpBridgeFilterChain() {
    return new FilterChain(MOUNT_POINT,
        AstrumAuthenticationFilter.NAME,
        ApiKeyAuthenticationFilter.NAME,
        AnonymousFilter.NAME,
        AntiCsrfFilter.NAME);
  }

  @ConditionalOnProperty(value = JWT_ENABLED, havingValue = "true")
  @Bean
  public FilterChain httpBridgeFilterChain_jwt() {
    return new FilterChain(MOUNT_POINT,
        AstrumAuthenticationFilter.NAME,
        JwtFilter.NAME,
        ApiKeyAuthenticationFilter.NAME,
        AnonymousFilter.NAME,
        AntiCsrfFilter.NAME);
  }
}
