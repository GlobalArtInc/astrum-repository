
package org.globalart.astrum.internal.metrics;

import java.lang.reflect.Method;
import java.util.Map;

import com.codahale.metrics.Metric;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.core.Is.is;

public class ReadOnlyMetricSetTest
{
  @Test
  public void testMetrics_alwaysReturnDefaults() throws Exception {
    ReadOnlyMetricSet readOnlyMetricSet = new ReadOnlyMetricSet();
    Map<String, Metric> metrics = readOnlyMetricSet.getMetrics();

    assertThat(getMetricValue(metrics.get("enabled")), is(false));
    assertThat(getMetricValue(metrics.get("pending")), is(0));
    assertThat(getMetricValue(metrics.get("freezeTime")), is(0L));
  }

  private Object getMetricValue(final Metric metric) throws Exception {
    Method m = metric.getClass().getMethod("getValue");
    return m.invoke(metric);
  }
}
