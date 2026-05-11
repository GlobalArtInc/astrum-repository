
package org.globalart.astrum.rapture.internal;

import java.util.Collections;
import java.util.stream.Stream;

import org.globalart.astrum.common.app.ManagedLifecycle;
import org.globalart.astrum.common.node.NodeAccess;
import org.globalart.astrum.common.scheduling.PeriodicJobService;
import org.globalart.astrum.common.scheduling.PeriodicJobService.PeriodicJob;
import org.globalart.astrum.common.stateguard.Guarded;
import org.globalart.astrum.common.stateguard.StateGuardLifecycleSupport;
import org.globalart.astrum.systemchecks.NodeSystemCheckResult;
import org.globalart.astrum.systemchecks.SystemCheckService;

import com.codahale.metrics.health.HealthCheck.Result;
import com.codahale.metrics.health.HealthCheckRegistry;
import com.google.common.cache.CacheBuilder;
import com.google.common.cache.CacheLoader;
import com.google.common.cache.LoadingCache;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.app.ManagedLifecycle.Phase.SERVICES;
import static org.globalart.astrum.common.stateguard.StateGuardLifecycleSupport.State.STARTED;

/**
 * Refreshes and caches the state of system {@link com.codahale.metrics.health.HealthCheck}s at the configured time
 * interval.
 *
 * @since 3.16
 */
@Component
@ManagedLifecycle(phase = SERVICES)
public class LocalSystemCheckService
    extends StateGuardLifecycleSupport
    implements SystemCheckService
{
  private final PeriodicJobService jobService;

  private final NodeAccess nodeAccess;

  private String hostname;

  private LoadingCache<String, Result> cache;

  private HealthCheckRegistry registry;

  private int refreshInterval;

  private PeriodicJob metricsWritingJob;

  @Autowired
  public LocalSystemCheckService(
      final PeriodicJobService jobService,
      final HealthCheckRegistry registry,
      final NodeAccess nodeAccess,
      @Value("${astrum.healthcheck.refreshInterval:15}") final int refreshInterval)
  {
    this.jobService = checkNotNull(jobService);
    this.registry = checkNotNull(registry);
    this.nodeAccess = checkNotNull(nodeAccess);
    this.refreshInterval = refreshInterval;
  }

  @Override
  protected void doStart() throws Exception {
    cache = CacheBuilder.newBuilder().build(cacheLoader());
    jobService.startUsing();

    // Retrieving the hostname may be expensive so do this asynchronously
    nodeAccess.getHostName()
        .thenAccept(value -> hostname = value)
        .exceptionally(ex -> {
          hostname = nodeAccess.getId();
          return null;
        });

    metricsWritingJob = jobService.schedule(this::refresh, refreshInterval);
  }

  @Override
  protected void doStop() throws Exception {
    metricsWritingJob.cancel();
    metricsWritingJob = null;
    jobService.stopUsing();
    cache = null;
  }

  private void refresh() {
    registry.getNames().forEach(this::refresh);
  }

  private void refresh(final String metricName) {
    LoadingCache<String, Result> localCache = this.cache;

    if (localCache == null) {
      // this case may happen during system shutdown
      return;
    }

    Result oldResult = localCache.getUnchecked(metricName);
    localCache.refresh(metricName);
    // Refresh is lazy and doesn't refresh until the next request so force it to refresh.
    Result newResult = localCache.getUnchecked(metricName);
    if (oldResult.isHealthy() != newResult.isHealthy()) {
      log.info("Health check status changed from {} to {} for {}", oldResult.isHealthy(), newResult.isHealthy(),
          metricName);
    }
  }

  private CacheLoader<String, Result> cacheLoader() {
    return new CacheLoader<String, Result>()
    {
      @Override
      public Result load(final String s) {
        return registry.runHealthCheck(s);
      }
    };
  }

  @Override
  @Guarded(by = STARTED)
  public Stream<NodeSystemCheckResult> getResults() {
    return Collections.singleton(new NodeSystemCheckResult(nodeAccess.getId(), hostname, cache.asMap())).stream();
  }
}
