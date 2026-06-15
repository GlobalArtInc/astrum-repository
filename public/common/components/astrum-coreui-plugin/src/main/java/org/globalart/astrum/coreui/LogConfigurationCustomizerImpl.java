
package org.globalart.astrum.coreui;

import org.globalart.astrum.common.log.LogConfigurationCustomizer;
import org.globalart.astrum.common.log.LoggerLevel;
import org.springframework.stereotype.Component;

/**
 * Core UI {@link LogConfigurationCustomizer}.
 *
 * @since 3.0
 */
@Component
public class LogConfigurationCustomizerImpl
    implements LogConfigurationCustomizer
{
  @Override
  public void customize(final Configuration configuration) {
    configuration.setLoggerLevel("org.globalart.astrum.coreui", LoggerLevel.DEFAULT);
  }
}
