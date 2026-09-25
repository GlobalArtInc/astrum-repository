
package com.globalart.astrum.ssl.plugin.internal;

import org.globalart.astrum.common.log.LogConfigurationCustomizer;
import org.globalart.astrum.common.log.LoggerLevel;
import org.springframework.stereotype.Component;

/**
 * Configures SSL loggers.
 *
 * @since 2.7
 */
@Component
public class LogConfigurationCustomizerImpl
    implements LogConfigurationCustomizer
{
  @Override
  public void customize(final Configuration configuration) {
    configuration.setLoggerLevel("com.globalart.astrum.ssl", LoggerLevel.DEFAULT);
  }
}
