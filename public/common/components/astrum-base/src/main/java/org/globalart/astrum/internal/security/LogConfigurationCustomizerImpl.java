
package org.globalart.astrum.internal.security;

import org.globalart.astrum.common.log.LogConfigurationCustomizer;
import org.globalart.astrum.common.log.LoggerLevel;
import org.springframework.stereotype.Component;

/**
 * Security {@link LogConfigurationCustomizer}.
 *
 * @since 3.0
 */
@Component
public class LogConfigurationCustomizerImpl
    implements LogConfigurationCustomizer
{
  @Override
  public void customize(final Configuration configuration) {
    configuration.setLoggerLevel("org.apache.shiro.web.filter.authc.BasicHttpAuthenticationFilter", LoggerLevel.INFO);
    configuration.setLoggerLevel("org.apache.shiro.web.filter.mgt.DefaultFilterChainManager", LoggerLevel.INFO);
    configuration.setLoggerLevel("org.globalart.astrum.security", LoggerLevel.DEFAULT);
    configuration.setLoggerLevel("org.globalart.astrum.internal.security", LoggerLevel.DEFAULT);
  }
}
