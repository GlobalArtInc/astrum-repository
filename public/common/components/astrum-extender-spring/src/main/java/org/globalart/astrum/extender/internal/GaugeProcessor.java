
package org.globalart.astrum.extender.internal;

import java.lang.reflect.Method;

import com.codahale.metrics.annotation.Gauge;
import org.springframework.core.annotation.AnnotationUtils;
import org.springframework.stereotype.Component;

import static com.codahale.metrics.MetricRegistry.name;

/**
 * Provides annotation support for {@link Gauge} metrics.
 */
@Component
public class GaugeProcessor
    extends AbstractGaugeProcessor<Gauge>
{
  @Override
  protected void registerMetric(Object bean, Method method, Gauge annotation, String metricName) {
    registerMetricAsGauge(bean, metricName, method);
  }

  @Override
  protected Gauge getAnnotation(Method method) {
    return AnnotationUtils.getAnnotation(method, Gauge.class);
  }

  @Override
  protected String metricName(Method method, Gauge gauge) {
    if (gauge.absolute()) {
      return gauge.name();
    }
    if (gauge.name().isEmpty()) {
      return name(method.getDeclaringClass(), method.getName(), "gauge");
    }
    return name(method.getDeclaringClass(), gauge.name());
  }

  @Override
  protected String gaugeName() {
    return "@Gauge";
  }
}
