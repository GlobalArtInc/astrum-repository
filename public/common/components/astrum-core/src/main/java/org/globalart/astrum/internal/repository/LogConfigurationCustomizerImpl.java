
package org.globalart.astrum.internal.repository;

import org.globalart.astrum.common.log.LogConfigurationCustomizer;

import static org.globalart.astrum.common.log.LoggerLevel.DEFAULT;
import org.springframework.stereotype.Component;

/**
 * Repository {@link LogConfigurationCustomizer}.
 *
 * @since 3.0
 */
@Component
public class LogConfigurationCustomizerImpl
    implements LogConfigurationCustomizer
{
  @Override
  public void customize(final Configuration config) {
    config.setLoggerLevel("org.globalart.astrum.repository", DEFAULT);
  }
}
