
package org.globalart.astrum.upgrade.datastore.internal;

import org.globalart.astrum.common.log.LogConfigurationCustomizer;
import org.globalart.astrum.common.log.LoggerLevel;
import org.springframework.stereotype.Component;

/**
 * Configures Flyway loggers.
 *
 * @since 3.29
 */
@Component
public class FlywayLogConfigurationCustomizer
    implements LogConfigurationCustomizer
{

  @Override
  public void customize(final Configuration configuration) {
    // limit noisy flyway loggers
    configuration.setLoggerLevel("org.flywaydb.core.internal", LoggerLevel.ERROR);
  }
}
