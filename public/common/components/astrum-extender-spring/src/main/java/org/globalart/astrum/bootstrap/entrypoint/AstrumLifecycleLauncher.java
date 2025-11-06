
package org.globalart.astrum.bootstrap.entrypoint;

import java.lang.management.ManagementFactory;
import java.util.EnumSet;
import java.util.Optional;
import java.util.Set;

import org.globalart.astrum.bootstrap.entrypoint.edition.AstrumEditionSelector;
import org.globalart.astrum.bootstrap.entrypoint.jvm.ShutdownDelegate;
import org.globalart.astrum.bootstrap.spring.AstrumComponentScanCompleteEvent;
import org.globalart.astrum.common.app.ApplicationVersion;
import org.globalart.astrum.common.app.ManagedLifecycle.Phase;
import org.globalart.astrum.extender.AstrumLifecycleManager;

import com.codahale.metrics.SharedMetricRegistries;
import com.google.common.base.Strings;
import jakarta.annotation.PreDestroy;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.context.ApplicationContext;
import org.springframework.context.event.EventListener;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.app.ManagedLifecycle.Phase.KERNEL;
import static org.globalart.astrum.common.app.ManagedLifecycle.Phase.OFF;
import static org.globalart.astrum.common.app.ManagedLifecycle.Phase.TASKS;
import static org.globalart.astrum.common.time.DateHelper.toDurationString;

/**
 * Waits for Jetty to start and component scan to complete before launching the astrum lifecycle.
 */
@Component
public class AstrumLifecycleLauncher
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final AstrumEditionSelector astrumEditionSelector;

  private final ShutdownDelegate shutdownDelegate;

  private final ApplicationVersion applicationVersion;

  private Phase startupPhase;

  private AstrumLifecycleManager astrumLifecycleManager;

  public AstrumLifecycleLauncher(
      final AstrumEditionSelector astrumEditionSelector,
      final ShutdownDelegate shutdownDelegate,
      @Autowired(required = false) final ApplicationVersion applicationVersion,
      @Value("${astrum.lifecycle.startupPhase:#{null}}") final String startupPhaseName)
  {
    this.astrumEditionSelector = checkNotNull(astrumEditionSelector);
    this.shutdownDelegate = checkNotNull(shutdownDelegate);
    this.applicationVersion = applicationVersion; // may be null
    // set startup phase from properties, if available
    Optional.ofNullable(startupPhaseName)
        .filter(value -> !Strings.isNullOrEmpty(value))
        .map(Phase::valueOf)
        .ifPresent((phase -> startupPhase = phase));
  }

  @PreDestroy
  public void preDestroy() {
    SharedMetricRegistries.remove("astrum");
  }

  public void stop() throws Exception {
    long uptime = ManagementFactory.getRuntimeMXBean().getUptime();
    log.info("Stopping Astrum, Uptime: {} ({})", toDurationString(uptime),
        EditionVersionFormatter.formatEditionAndVersion(astrumEditionSelector, applicationVersion));
    if (astrumLifecycleManager != null) {
      astrumLifecycleManager.to(Phase.OFF);
    }
  }

  @EventListener
  public void onAstrumScanComplete(final AstrumComponentScanCompleteEvent event) throws Exception {
    log.debug("Astrum component scan completed - edition ({})", astrumEditionSelector.getCurrent().getShortName());
    maybeLaunch((ApplicationContext) event.getSource());
  }

  private void maybeLaunch(final ApplicationContext context) throws Exception {
    logStartupPhase();
    // We use the event context to retrieve AstrumLifecycleManager as it is from the second round of scanning
    astrumLifecycleManager = context.getBean(AstrumLifecycleManager.class);

    if (astrumLifecycleManager == null) {
      log.debug("received AstrumComponentScanCompleteEvent but no AstrumLifecycleManager available");
      return;
    }
    try {
      // on startup we want to move to the TASKS phase
      moveToPhase(TASKS);
    }
    catch (Exception e) {
      log.error("Failed to move to phase TASKS: {}", e.getMessage(), e);
      shutdownDelegate.exit(1);
    }
  }

  /**
   * Log the startup phases that will be executed.
   */
  private void logStartupPhase() {
    Set<Phase> phases = EnumSet.complementOf(EnumSet.of(OFF));

    if (startupPhase != null) {
      phases = EnumSet.range(KERNEL, startupPhase);
    }

    log.info("Running lifecycle phases {}", phases);
  }

  /**
   * Move to the given phase, but only if it is not past the startup phase.
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
