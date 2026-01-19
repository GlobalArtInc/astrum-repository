
package org.globalart.astrum.extdirect;

import org.globalart.astrum.extdirect.internal.ExtDirectServlet;
import org.globalart.astrum.security.FilterChain;
import org.globalart.astrum.security.anonymous.AnonymousFilter;
import org.globalart.astrum.security.authc.AntiCsrfFilter;
import org.globalart.astrum.security.authc.AstrumAuthenticationFilter;

import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Configuration
public class ExtDirectConfiguration
{
  private static final String MOUNT_POINT = ExtDirectServlet.PREFIXED_MOUNT_POINT + "*";

  @Bean
  public FilterChain extDirectFilterChain() {
    return new FilterChain(MOUNT_POINT, AstrumAuthenticationFilter.NAME, AnonymousFilter.NAME, AntiCsrfFilter.NAME);
  }
}
