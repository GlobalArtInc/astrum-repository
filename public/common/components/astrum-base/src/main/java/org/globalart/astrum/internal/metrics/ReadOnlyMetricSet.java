
package org.globalart.astrum.internal.metrics;

import java.util.Map;

import com.codahale.metrics.Gauge;
import com.codahale.metrics.Metric;
import com.codahale.metrics.MetricSet;
import com.google.common.collect.ImmutableMap;

/**
 * {@link Metric} providing information about the database readonly (frozen) status.
 * Since freeze functionality has been removed, these metrics always return default values.
 *
 * @since 3.6
 */
public class ReadOnlyMetricSet
    implements MetricSet
{
  private final Map<String, Metric> metrics;

  public ReadOnlyMetricSet() {
    this.metrics = ImmutableMap.of(
        "enabled", (Gauge<Boolean>) () -> false,
        "pending", (Gauge<Integer>) () -> 0,
        "freezeTime", (Gauge<Long>) () -> 0L);
  }

  @Override
  public Map<String, Metric> getMetrics() {
    return metrics;
  }
}
