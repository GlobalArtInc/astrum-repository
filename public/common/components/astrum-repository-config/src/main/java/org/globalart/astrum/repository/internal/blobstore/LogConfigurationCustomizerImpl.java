
package org.globalart.astrum.repository.internal.blobstore;

import org.globalart.astrum.common.log.LogConfigurationCustomizer;

import static org.globalart.astrum.common.log.LoggerLevel.DEFAULT;
import org.springframework.stereotype.Component;

/**
 * Blob-store {@link LogConfigurationCustomizer}.
 *
 * @since 3.0
 */
@Component
public class LogConfigurationCustomizerImpl
    implements LogConfigurationCustomizer
{
  @Override
  public void customize(final Configuration config) {
    config.setLoggerLevel("org.globalart.astrum.blobstore", DEFAULT);
    config.setLoggerLevel("org.globalart.astrum.blobstore.iostat", DEFAULT);
    config.setLoggerLevel("org.globalart.astrum.repository.internal.blobstore", DEFAULT);
  }
}
