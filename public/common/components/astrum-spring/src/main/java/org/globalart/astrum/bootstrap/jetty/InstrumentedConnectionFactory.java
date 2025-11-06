
package org.globalart.astrum.bootstrap.jetty;

import java.util.List;

import com.codahale.metrics.MetricRegistry;
import com.codahale.metrics.SharedMetricRegistries;
import org.eclipse.jetty.server.ConnectionFactory;

import static org.globalart.astrum.common.app.FeatureFlags.METRICS_INTERNAL_ENABLED;
import static org.globalart.astrum.common.metrics.MetricsConstants.ASTRUM_METRICS_REGISTRY_NAME;

/**
 * Extension of {@link io.dropwizard.metrics.jetty12.InstrumentedConnectionFactory}.
 */
public final class InstrumentedConnectionFactory // NOSONAR
    extends io.dropwizard.metrics.jetty12.InstrumentedConnectionFactory
{
  private static final MetricRegistry NOOP_REGISTRY = new MetricRegistry();

  private final ConnectionFactory connectionFactory;

  public InstrumentedConnectionFactory(final ConnectionFactory connectionFactory) {
    super(connectionFactory, resolveRegistry().timer("connection-duration"));
    this.connectionFactory = connectionFactory;
  }

  private static MetricRegistry resolveRegistry() {
    if (Boolean.parseBoolean(System.getProperty(METRICS_INTERNAL_ENABLED, "true"))) {
      return SharedMetricRegistries.getOrCreate(ASTRUM_METRICS_REGISTRY_NAME);
    }
    return NOOP_REGISTRY;
  }

  // HACK: metrics-jetty9 (presently) is based on jetty 9.2, but we have to add more api for jetty 9.3

  @Override
  public List<String> getProtocols() {
    return connectionFactory.getProtocols();
  }
}
