
package org.globalart.astrum.internal.log;

import org.globalart.astrum.common.log.LogConfigurationCustomizer;
import org.globalart.astrum.common.log.LoggerLevel;

import org.springframework.stereotype.Component;

/**
 * Configures core Astrum loggers.
 *
 * @since 2.7
 */
@Component
public class LogConfigurationCustomizerImpl
    implements LogConfigurationCustomizer
{
  @Override
  public void customize(final Configuration configuration) {
    // non Astrum loggers
    configuration.setLoggerLevel("org.apache.commons", LoggerLevel.WARN);

    configuration.setLoggerLevel("org.eclipse.jetty", LoggerLevel.INFO);
    configuration.setLoggerLevel("eu.medsea.mimeutil.MimeUtil2", LoggerLevel.INFO);

    // ASTRUM-5835: limit noisy jmx connections to Astrum when root.level is DEBUG
    configuration.setLoggerLevel("javax.management", LoggerLevel.INFO);
    configuration.setLoggerLevel("sun.rmi", LoggerLevel.INFO);

    // Useful loggers (level will be calculated as effective level)
    configuration.setLoggerLevel("org.globalart.astrum", LoggerLevel.DEFAULT);

    configuration.setLoggerLevel("org.globalart.astrum.jmx", LoggerLevel.DEFAULT);
    configuration.setLoggerLevel("org.globalart.astrum.internal.log", LoggerLevel.DEFAULT);
    configuration.setLoggerLevel("org.globalart.astrum.plugins", LoggerLevel.DEFAULT);
  }
}
