
package org.globalart.astrum.quartz.internal;

import org.globalart.astrum.common.log.LogConfigurationCustomizer;
import org.globalart.astrum.common.log.LoggerLevel;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Quartz {@link LogConfigurationCustomizer}.
 *
 * @since 3.0
 */
@Component
public class LogConfigurationCustomizerImpl
    implements LogConfigurationCustomizer
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  @Override
  public void customize(final LogConfigurationCustomizer.Configuration configuration) {
    configuration.setLoggerLevel("org.globalart.astrum.quartz", LoggerLevel.DEFAULT);

    // Quartz is chatty at INFO
    configuration.setLoggerLevel("org.quartz", LoggerLevel.WARN);
  }
}
