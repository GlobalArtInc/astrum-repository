
package org.globalart.astrum.datastore.h2;

import java.io.File;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.bootstrap.entrypoint.configuration.ApplicationDirectories;
import org.globalart.astrum.common.app.ManagedLifecycle;
import org.globalart.astrum.common.event.EventAware;
import org.globalart.astrum.common.node.NodeAccess;
import org.globalart.astrum.common.stateguard.Guarded;
import org.globalart.astrum.common.stateguard.StateGuardLifecycleSupport;

import org.h2.tools.Server;
import org.springframework.beans.factory.annotation.Value;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.app.ManagedLifecycle.Phase.TASKS;
import static org.globalart.astrum.common.stateguard.StateGuardLifecycleSupport.State.STARTED;
import org.springframework.stereotype.Component;

/**
 * Enable H2 web console.
 *
 * @since 3.27
 */
@Component
@ManagedLifecycle(phase = TASKS)
public class H2WebConsole
    extends StateGuardLifecycleSupport
    implements EventAware, EventAware.Asynchronous
{
  private final File databasesDir;

  private final boolean httpListenerEnabled;

  private final int httpListenerPort;

  private Server h2Server;

  @Autowired
  public H2WebConsole(
      final ApplicationDirectories applicationDirectories,
      @Value("${astrum.h2.httpListenerEnabled:false}") final boolean httpListenerEnabled,
      @Value("${astrum.h2.httpListenerPort:8082}") final int httpListenerPort,
      final NodeAccess nodeAccess)
  {
    checkNotNull(applicationDirectories);
    this.httpListenerEnabled = httpListenerEnabled;
    this.httpListenerPort = httpListenerPort;
    databasesDir = applicationDirectories.getWorkDirectory("db", false);
  }

  @Override
  protected void doStart() throws Exception {
    if (httpListenerEnabled) {
      h2Server = Server.createWebServer("-webPort", String.valueOf(httpListenerPort), "-webAllowOthers", "-ifExists",
          "-baseDir", databasesDir.toString()).start();
      log.info("Activated");
    }
  }

  @Override
  protected void doStop() throws Exception {
    if (h2Server != null) {
      // instance shutdown
      h2Server.shutdown();
      h2Server = null;

      log.info("Shutdown");
    }
  }

  @Guarded(by = STARTED)
  public Server getH2Server() {
    return h2Server;
  }
}
