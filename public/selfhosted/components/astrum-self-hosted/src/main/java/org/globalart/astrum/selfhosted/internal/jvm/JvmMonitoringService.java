
package org.globalart.astrum.selfhosted.internal.jvm;

import java.time.Duration;
import java.util.List;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.common.app.ManagedLifecycle;
import org.globalart.astrum.common.scheduling.PeriodicJobService;
import org.globalart.astrum.common.scheduling.PeriodicJobService.PeriodicJob;
import org.globalart.astrum.common.stateguard.StateGuardLifecycleSupport;

import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.app.ManagedLifecycle.Phase.SERVICES;

/**
 * JVM Monitoring Service that orchestrates periodic monitoring of JVM health.
 *
 * This service coordinates multiple JVM monitors (GC, memory, etc.) and executes
 * them periodically to log warnings when thresholds are exceeded.
 *
 * Replaces the monitoring functionality previously provided by embedded Elasticsearch.
 */
@Component
@ManagedLifecycle(phase = SERVICES)
public class JvmMonitoringService
    extends StateGuardLifecycleSupport
{
  private final PeriodicJobService periodicJobService;

  private final List<JvmMonitor> monitors;

  private final boolean enabled;

  private final int checkIntervalSeconds;

  private PeriodicJob monitoringJob;

  @Autowired
  public JvmMonitoringService(
      final PeriodicJobService periodicJobService,
      final List<JvmMonitor> monitors,
      @Value("${astrum.jvm.monitor.enabled:true}") final boolean enabled,
      @Value("${astrum.jvm.monitor.interval:15}") final int checkIntervalSeconds)
  {
    this.periodicJobService = checkNotNull(periodicJobService);
    this.monitors = checkNotNull(monitors);
    this.enabled = enabled;
    this.checkIntervalSeconds = checkIntervalSeconds;
  }

  @Override
  protected void doStart() throws Exception {
    if (!enabled) {
      log.info("JVM monitoring is disabled");
      return;
    }

    log.info("Starting JVM monitoring service (check interval: {}s, {} monitors registered)",
        checkIntervalSeconds, monitors.size());

    // Initialize all monitors
    for (JvmMonitor monitor : monitors) {
      monitor.initialize();
    }

    periodicJobService.startUsing();
    monitoringJob = periodicJobService.schedule(
        this::checkJvmHealth,
        Duration.ofSeconds(checkIntervalSeconds),
        Duration.ofSeconds(checkIntervalSeconds));
  }

  @Override
  protected void doStop() throws Exception {
    if (!enabled) {
      return;
    }

    log.info("Stopping JVM monitoring service");
    if (monitoringJob != null) {
      monitoringJob.cancel();
      monitoringJob = null;
    }
    periodicJobService.stopUsing();

    // Cleanup all monitors
    for (JvmMonitor monitor : monitors) {
      monitor.cleanup();
    }
  }

  private void checkJvmHealth() {
    try {
      // Execute all registered monitors
      for (JvmMonitor monitor : monitors) {
        monitor.check();
      }
    }
    catch (Exception e) {
      // Don't propagate exceptions as they would cancel the periodic job
      log.error("JVM monitoring check failed", e);
    }
  }
}
