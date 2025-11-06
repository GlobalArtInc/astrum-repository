
package org.globalart.astrum.bootstrap.jetty;

import org.globalart.astrum.bootstrap.entrypoint.jetty.JettyServer;

import org.slf4j.MDC;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.context.ApplicationContext;
import org.springframework.context.SmartLifecycle;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Ties into Springs lifecycle phase to manage Astrum startup & shutdown
 */
@Component
public class ManagedJetty
    implements SmartLifecycle
{
  public static final String SYSTEM_USERID = "*SYSTEM";

  private final JettyServer jetty;

  private final ApplicationContext context;

  private boolean started;

  @Autowired
  public ManagedJetty(final ApplicationContext context, final JettyServer jetty) {
    this.context = checkNotNull(context);
    this.jetty = checkNotNull(jetty);
  }

  @Override
  public void start() {
    try {
      MDC.put("userId", SYSTEM_USERID);
      jetty.start(context, true);
    }
    catch (Exception e) {
      if (e instanceof RuntimeException re) {
        throw re;
      }
      throw new RuntimeException(e);
    }
    started = true;
  }

  @Override
  public void stop() {
    try {
      jetty.stop();
    }
    catch (Exception e) {
      if (e instanceof RuntimeException re) {
        throw re;
      }
      throw new RuntimeException(e);
    }
    finally {
      MDC.remove("userId");
    }
    started = false;
  }

  @Override
  public int getPhase() {
    // give Astrum a low precedence to ensure any Spring core is already complete
    return Integer.MAX_VALUE;
  }

  @Override
  public boolean isRunning() {
    return started;
  }

}
