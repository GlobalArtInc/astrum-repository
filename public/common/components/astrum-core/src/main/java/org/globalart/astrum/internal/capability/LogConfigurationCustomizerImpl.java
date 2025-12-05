
package org.globalart.astrum.internal.capability;

import org.globalart.astrum.common.log.LogConfigurationCustomizer;
import org.globalart.astrum.common.log.LoggerLevel;
import org.springframework.stereotype.Component;

/**
 * Capabilities {@link LogConfigurationCustomizer}.
 *
 * @since 2.7
 */
@Component
public class LogConfigurationCustomizerImpl
    implements LogConfigurationCustomizer
{
  @Override
  public void customize(final Configuration configuration) {
    // Useful loggers (level will be calculated as effective level)
    configuration.setLoggerLevel("org.globalart.astrum.capability", LoggerLevel.DEFAULT);
  }
}
