
package org.globalart.astrum.internal.metrics;

import com.codahale.metrics.Metric;
import org.springframework.beans.factory.FactoryBean;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

/**
 * {@link Metric} provider for the database readonly (frozen) status.
 *
 * @since 3.6
 */
@Component
@Qualifier("readonly")
public class ReadOnlyMetricProvider
    implements FactoryBean<Metric>
{
  public Metric getObject() {
    return new ReadOnlyMetricSet();
  }

  @Override
  public Class<?> getObjectType() {
    return Metric.class;
  }
}
