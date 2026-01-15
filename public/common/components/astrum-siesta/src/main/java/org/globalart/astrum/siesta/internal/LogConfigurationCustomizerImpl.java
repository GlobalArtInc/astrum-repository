
package org.globalart.astrum.siesta.internal;

import org.globalart.astrum.common.log.LogConfigurationCustomizer;

import static org.globalart.astrum.common.log.LoggerLevel.DEFAULT;
import org.springframework.stereotype.Component;

/**
 * Siesta {@link LogConfigurationCustomizer}.
 *
 * @since 2.7
 */
@Component
public class LogConfigurationCustomizerImpl
    implements LogConfigurationCustomizer
{
  @Override
  public void customize(final Configuration configuration) {
    configuration.setLoggerLevel("org.globalart.astrum.rest", DEFAULT);
    configuration.setLoggerLevel("org.globalart.astrum.plugins.siesta", DEFAULT);
    configuration.setLoggerLevel("org.jboss.resteasy", DEFAULT);
  }
}
