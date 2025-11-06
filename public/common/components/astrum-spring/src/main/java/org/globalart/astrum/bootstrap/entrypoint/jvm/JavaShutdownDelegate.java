
package org.globalart.astrum.bootstrap.entrypoint.jvm;

import org.springframework.boot.autoconfigure.condition.ConditionalOnProperty;
import org.springframework.stereotype.Component;

@Component
@ConditionalOnProperty(value = "astrum.noop.shutdown.delegate", havingValue = "false", matchIfMissing = true)
public class JavaShutdownDelegate
    implements ShutdownDelegate
{
  @Override
  public void exit(final int code) {
    // expected use of System.exit()
    System.exit(code);
  }

  @Override
  public void halt(final int code) {
    // expected use of Runtime.halt()
    Runtime.getRuntime().halt(code);
  }
}
