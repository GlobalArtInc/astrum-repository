
package org.globalart.astrum.datastore.internal;

import org.globalart.astrum.common.log.LogConfigurationCustomizer;

import static org.globalart.astrum.common.log.LoggerLevel.DEFAULT;
import org.springframework.stereotype.Component;

/**
 * DataStore {@link LogConfigurationCustomizer}.
 *
 * @since 3.19
 */
@Component
public class LogConfigurationCustomizerImpl
    implements LogConfigurationCustomizer
{
  @Override
  public void customize(final Configuration config) {
    config.setLoggerLevel("org.globalart.astrum.datastore", DEFAULT);
  }
}
