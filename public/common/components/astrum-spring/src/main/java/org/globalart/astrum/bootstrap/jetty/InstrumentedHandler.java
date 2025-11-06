
package org.globalart.astrum.bootstrap.jetty;

import com.codahale.metrics.MetricRegistry;
import com.codahale.metrics.SharedMetricRegistries;
import org.eclipse.jetty.ee10.servlet.ServletContextHandler;
import org.eclipse.jetty.server.Request;
import org.eclipse.jetty.server.Response;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.app.FeatureFlags.METRICS_INTERNAL_ENABLED;
import static org.globalart.astrum.common.metrics.MetricsConstants.ASTRUM_METRICS_REGISTRY_NAME;

/**
 * Extension of {@link io.dropwizard.metrics.jetty12.AbstractInstrumentedHandler} that restores the delegate
 * constructor.
 */
public final class InstrumentedHandler // NOSONAR
    extends io.dropwizard.metrics.jetty12.AbstractInstrumentedHandler
{
  private static final MetricRegistry NOOP_REGISTRY = new MetricRegistry();

  private final ServletContextHandler delegate;

  public InstrumentedHandler(final ServletContextHandler delegate) {
    super(resolveRegistry());
    setHandler(delegate);
    this.delegate = checkNotNull(delegate);
  }

  private static MetricRegistry resolveRegistry() {
    if (Boolean.parseBoolean(System.getProperty(METRICS_INTERNAL_ENABLED, "true"))) {
      return SharedMetricRegistries.getOrCreate(ASTRUM_METRICS_REGISTRY_NAME);
    }
    return NOOP_REGISTRY;
  }

  public ServletContextHandler getDelegate() {
    return delegate;
  }

  @Override
  protected void setupServletListeners(final Request request, final Response response) {
    // no-op
  }

  @Override
  protected boolean isSuspended(final Request request, final Response response) {
    return false;
  }
}
