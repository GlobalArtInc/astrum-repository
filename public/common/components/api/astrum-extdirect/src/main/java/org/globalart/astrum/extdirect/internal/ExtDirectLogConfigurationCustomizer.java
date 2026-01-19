
package org.globalart.astrum.extdirect.internal;

import org.globalart.astrum.common.log.LogConfigurationCustomizer;
import org.globalart.astrum.common.log.LoggerLevel;
import org.springframework.stereotype.Component;

/**
 * Ext.Direct {@link LogConfigurationCustomizer}.
 *
 * @since 3.0
 */
@Component
public class ExtDirectLogConfigurationCustomizer
    implements LogConfigurationCustomizer
{

  @Override
  public void customize(final Configuration configuration) {
    // Useful loggers (level will be calculated as effective level)
    configuration.setLoggerLevel("org.globalart.astrum.extdirect", LoggerLevel.DEFAULT);
    configuration.setLoggerLevel("com.softwarementors.extjs.djn", LoggerLevel.DEFAULT);
  }

}
