
package org.globalart.astrum.common.log;

import com.codahale.metrics.SharedMetricRegistries;

import static org.globalart.astrum.common.metrics.MetricsConstants.ASTRUM_METRICS_REGISTRY_NAME;

/**
 * Extension of {@link com.codahale.metrics.logback.InstrumentedAppender} that restores the default constructor.
 * 
 * @since 3.0
 */
public final class InstrumentedAppender
    extends com.codahale.metrics.logback.InstrumentedAppender
{
  public InstrumentedAppender() {
    super(SharedMetricRegistries.getOrCreate(ASTRUM_METRICS_REGISTRY_NAME));
  }
}
