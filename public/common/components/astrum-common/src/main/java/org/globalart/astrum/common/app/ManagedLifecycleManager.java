
package org.globalart.astrum.common.app;

import org.globalart.astrum.common.app.ManagedLifecycle.Phase;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Manages {@link ManagedLifecycle} components.
 *
 * @since 3.3
 */
public abstract class ManagedLifecycleManager
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  /**
   * Returns the current phase.
   */
  public abstract Phase getCurrentPhase();

  /**
   * Attempts to move to the target phase by starting (or stopping) components phase-by-phase. If any components have
   * appeared since the last request which belong to the current phase or earlier then they are automatically started
   * before the current phase is changed. Similarly components that have disappeared are stopped.
   */
  public abstract void to(final Phase targetPhase) throws Exception;

  /**
   * Attempts to bounce the given phase by moving the lifecycle just before it then back towards the current phase,
   * re-running all the phases in between. If the bounce phase is after the current phase then it simply moves the
   * lifecycle forwards like {@link #to(Phase)}.
   *
   * @since 3.16
   */
  public abstract void bounce(final Phase bouncePhase) throws Exception;

  /**
   * Are we in the process of shutting down? (ie. moving to the {@code OFF} phase)
   *
   * @since 3.16
   */
  public static boolean isShuttingDown() {
    return shuttingDown;
  }

  private static volatile boolean shuttingDown;

  protected ManagedLifecycleManager() {
    shuttingDown = false;
  }

  /**
   * Flag that we are in the process of shutting down.
   *
   * @since 3.16
   */
  protected void declareShutdown() {
    log.info("Shutting down");
    shuttingDown = true;
  }

  /**
   * Shutdown Astrum, and provide a custom exit code to the calling system/process. This should ensure that all services
   * and phases are stopped and safe before ending.
   * 
   * @param exitCode the exit code to provide to the calling system/process
   * @throws Exception the lifecycle manager may propagate exceptions if the change is not possible
   */
  public void shutdownWithExitCode(final int exitCode) throws Exception {
    System.setProperty("astrum.overrideExitCode", Integer.toString(exitCode));
    log.info("Shutdown requested with an exit code of " + exitCode);
    this.to(Phase.OFF);
  }
}
