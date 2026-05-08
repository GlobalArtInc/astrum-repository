
package org.globalart.astrum.bootstrap.rapture;

import org.globalart.astrum.rapture.internal.security.JwtAuthenticationFilter;
import org.globalart.astrum.rapture.internal.security.SessionAuthenticationFilter;
import org.globalart.astrum.security.FilterChain;

import org.springframework.boot.autoconfigure.condition.ConditionalOnProperty;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

import static org.globalart.astrum.common.app.FeatureFlags.JWT_ENABLED;
import static org.globalart.astrum.common.app.FeatureFlags.SESSION_ENABLED;
import static org.globalart.astrum.rapture.internal.security.SessionServlet.SESSION_MP;

@Configuration
public class RaptureConfiguration
{
  @ConditionalOnProperty(value = SESSION_ENABLED, havingValue = "true", matchIfMissing = true)
  @Bean
  public FilterChain sessionFilterChain() {
    return new FilterChain(SESSION_MP, SessionAuthenticationFilter.NAME);
  }

  @ConditionalOnProperty(value = JWT_ENABLED, havingValue = "true")
  @Bean
  public FilterChain sessionFilterChain_jwt() {
    return new FilterChain(SESSION_MP, JwtAuthenticationFilter.NAME);
  }

}
