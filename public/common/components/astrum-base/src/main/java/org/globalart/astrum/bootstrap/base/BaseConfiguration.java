
package org.globalart.astrum.bootstrap.base;

import org.globalart.astrum.security.FilterChain;
import org.globalart.astrum.security.JwtFilter;
import org.globalart.astrum.security.anonymous.AnonymousFilter;
import org.globalart.astrum.security.authc.AntiCsrfFilter;

import com.codahale.metrics.Clock;
import org.springframework.boot.autoconfigure.condition.ConditionalOnProperty;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.core.Ordered;
import org.springframework.core.annotation.Order;

import static org.globalart.astrum.common.app.FeatureFlags.JWT_ENABLED;
import static org.globalart.astrum.common.app.FeatureFlags.SESSION_ENABLED;

/**
 * Web resources module. Both servlet and filter-chain are installed with the lowest priority.
 */
@Configuration
public class BaseConfiguration
{

  @Bean("astrum-uber")
  public ClassLoader uberClassLoader() {
    return BaseConfiguration.class.getClassLoader();
  }

  @Bean
  public com.codahale.metrics.Clock clock() {
    return Clock.defaultClock();
  }

  @Order(Ordered.LOWEST_PRECEDENCE)
  @ConditionalOnProperty(value = SESSION_ENABLED, havingValue = "true", matchIfMissing = true)
  @Bean
  public FilterChain primaryFilterChain() {
    return new FilterChain("/**", AnonymousFilter.NAME, AntiCsrfFilter.NAME);
  }

  @Order(Ordered.LOWEST_PRECEDENCE)
  @ConditionalOnProperty(value = JWT_ENABLED, havingValue = "true")
  @Bean
  public FilterChain primaryFilterChain_jwt() {
    return new FilterChain("/**", JwtFilter.NAME, AnonymousFilter.NAME, AntiCsrfFilter.NAME);
  }
}
