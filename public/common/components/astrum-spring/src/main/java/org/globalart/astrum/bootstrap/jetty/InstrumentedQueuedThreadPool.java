
package org.globalart.astrum.bootstrap.jetty;

import com.codahale.metrics.MetricRegistry;
import com.codahale.metrics.SharedMetricRegistries;

import static org.globalart.astrum.common.app.FeatureFlags.METRICS_INTERNAL_ENABLED;
import static org.globalart.astrum.common.metrics.MetricsConstants.ASTRUM_METRICS_REGISTRY_NAME;

/**
 * Extension of {@link io.dropwizard.metrics.jetty12.InstrumentedQueuedThreadPool} that restores the default
 * constructor.
 */
public final class InstrumentedQueuedThreadPool // NOSONAR
    extends io.dropwizard.metrics.jetty12.InstrumentedQueuedThreadPool
{
  private static final MetricRegistry NOOP_REGISTRY = new MetricRegistry();

  public InstrumentedQueuedThreadPool() {
    super(resolveRegistry());
  }

  private static MetricRegistry resolveRegistry() {
    if (Boolean.parseBoolean(System.getProperty(METRICS_INTERNAL_ENABLED, "true"))) {
      return SharedMetricRegistries.getOrCreate(ASTRUM_METRICS_REGISTRY_NAME);
    }
    return NOOP_REGISTRY;
  }
}
