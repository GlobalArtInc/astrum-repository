
package org.globalart.astrum.audit.internal;

import org.globalart.astrum.common.log.LogConfigurationCustomizer;
import org.globalart.astrum.common.log.LoggerLevel;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Audit {@link LogConfigurationCustomizer}.
 */
@Component
public class LogConfigurationCustomizerImpl
    implements LogConfigurationCustomizer
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  @Override
  public void customize(final Configuration configuration) {
    configuration.setLoggerLevel("org.globalart.astrum.audit", LoggerLevel.DEFAULT);
  }
}
