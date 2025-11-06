
package org.globalart.astrum.extender.internal;

import java.lang.management.ManagementFactory;
import java.util.EnumSet;

import jakarta.servlet.ServletContextEvent;
import jakarta.servlet.ServletContextListener;

import org.globalart.astrum.bootstrap.entrypoint.EditionVersionFormatter;
import org.globalart.astrum.bootstrap.entrypoint.edition.AstrumEditionSelector;
import org.globalart.astrum.common.app.ApplicationVersion;
import org.globalart.astrum.common.app.ManagedLifecycle.Phase;
import org.globalart.astrum.extender.AstrumLifecycleManager;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.context.annotation.Lazy;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.app.ManagedLifecycle.Phase.KERNEL;
import static org.globalart.astrum.common.app.ManagedLifecycle.Phase.OFF;
import static org.globalart.astrum.common.app.ManagedLifecycle.Phase.TASKS;
import static org.globalart.astrum.common.time.DateHelper.toDurationString;

@Lazy
@Component
public class AstrumServletContextListener
    implements ServletContextListener
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private static final String ASTRUM_LIFECYCLE_STARTUP_PHASE = "astrum.lifecycle.startupPhase";

  private final AstrumEditionSelector astrumEditionSelector;

  private final AstrumLifecycleManager astrumLifecycleManager;

  private final ApplicationVersion applicationVersion;

  private Phase startupPhase;

  @Autowired
  public AstrumServletContextListener(
      final AstrumEditionSelector astrumEditionSelector,
      final AstrumLifecycleManager astrumLifecycleManager,
      @Autowired(required = false) final ApplicationVersion applicationVersion,
      @Value("${" + ASTRUM_LIFECYCLE_STARTUP_PHASE + ":#{null}}") final String startupPhaseValue)
  {
    this.astrumLifecycleManager = checkNotNull(astrumLifecycleManager);
    this.astrumEditionSelector = checkNotNull(astrumEditionSelector);
    this.applicationVersion = applicationVersion; // may be null
    if (startupPhaseValue != null) {
      startupPhase = Phase.valueOf(startupPhaseValue);
    }
  }

  @Override
  public void contextInitialized(final ServletContextEvent event) {
    try {
      checkStartupPhase();
      // Push to the last phase in lifecycle, this will of course process each phase in between en route to TASKS phase
      moveToPhase(TASKS);
    }
    catch (final Exception e) {
      log.error("Failed to initialize context", e);
      throw new RuntimeException(e);
    }
  }

  @Override
  public void contextDestroyed(final ServletContextEvent event) {
    // log uptime before triggering activity which may run into problems
    long uptime = ManagementFactory.getRuntimeMXBean().getUptime();
    log.info("Uptime: {} ({})", toDurationString(uptime),
        EditionVersionFormatter.formatEditionAndVersion(astrumEditionSelector, applicationVersion));

    try {
      moveToPhase(OFF);
    }
    catch (final Exception e) {
      log.error("Failed to stop astrum", e);
    }
  }

  /**
   * Checks whether we should limit application startup to a particular lifecycle phase.
   */
  private void checkStartupPhase() {
    if (startupPhase != null) {
      log.info("Running lifecycle phases {}", EnumSet.range(KERNEL, startupPhase));
    }
    else {
      log.info("Running lifecycle phases {}", EnumSet.complementOf(EnumSet.of(OFF)));
    }
  }

  /**
   * Moves the application lifecycle on to a new phase.
   * <p>
   * When {@link #startupPhase} is set startup will never go past that phase.
   */
  private void moveToPhase(final Phase phase) throws Exception {
    if (startupPhase != null && phase.ordinal() > startupPhase.ordinal()) {
      astrumLifecycleManager.to(startupPhase); // this far, no further
    }
    else {
      astrumLifecycleManager.to(phase);
    }
  }
}
