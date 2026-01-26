
package org.globalart.astrum.blobstore.s3.internal;

import org.globalart.astrum.common.log.LogConfigurationCustomizer;
import org.globalart.astrum.common.log.LoggerLevel;
import org.springframework.stereotype.Component;

/**
 * Configures S3 loggers.
 *
 * @since 3.14
 */
@Component
public class S3LogConfigurationCustomizer
    implements LogConfigurationCustomizer
{

  @Override
  public void customize(final Configuration configuration) {
    // limit noisy S3 logger
    configuration.setLoggerLevel("software.amazon.awssdk.services.s3.internal.sync.S3AbortableInputStream",
        LoggerLevel.ERROR);
  }
}
