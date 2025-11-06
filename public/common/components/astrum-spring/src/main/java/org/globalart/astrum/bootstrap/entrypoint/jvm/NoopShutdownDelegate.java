
package org.globalart.astrum.bootstrap.entrypoint.jvm;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.boot.autoconfigure.condition.ConditionalOnProperty;
import org.springframework.stereotype.Component;

@Component
@ConditionalOnProperty(value = "astrum.noop.shutdown.delegate", havingValue = "true")
public class NoopShutdownDelegate
    implements ShutdownDelegate
{
  private static final Logger LOG = LoggerFactory.getLogger(NoopShutdownDelegate.class);

  @Override
  public void exit(final int code) {
    LOG.warn("Ignoring exit({}) request", code);
  }

  @Override
  public void halt(final int code) {
    LOG.warn("Ignoring halt({}) request", code);
  }
}
