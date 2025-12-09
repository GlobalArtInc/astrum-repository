
package org.globalart.astrum.internal.httpclient;

import org.globalart.astrum.common.log.LogConfigurationCustomizer;
import org.globalart.astrum.common.log.LoggerLevel;

import static org.globalart.astrum.internal.httpclient.HttpClientManagerImpl.HTTPCLIENT_OUTBOUND_LOGGER_NAME;
import org.springframework.stereotype.Component;

/**
 * HTTP-client {@link LogConfigurationCustomizer}.
 *
 * @since 3.0
 */
@Component
public class LogConfigurationCustomizerImpl
    implements LogConfigurationCustomizer
{
  @Override
  public void customize(final Configuration config) {
    // ASTRUM-6134: make it easy for user to debug outbound request headers
    config.setLoggerLevel("org.apache.http", LoggerLevel.INFO);
    config.setLoggerLevel("org.apache.http.wire", LoggerLevel.ERROR);

    config.setLoggerLevel("org.globalart.astrum.httpclient", LoggerLevel.DEFAULT);
    config.setLoggerLevel("org.globalart.astrum.internal.httpclient", LoggerLevel.DEFAULT);

    config.setLoggerLevel(HTTPCLIENT_OUTBOUND_LOGGER_NAME, LoggerLevel.DEFAULT);
  }
}
