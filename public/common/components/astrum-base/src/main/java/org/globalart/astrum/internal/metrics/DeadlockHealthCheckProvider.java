
package org.globalart.astrum.internal.metrics;

import com.codahale.metrics.health.HealthCheck;
import com.codahale.metrics.health.jvm.ThreadDeadlockHealthCheck;
import org.springframework.beans.factory.FactoryBean;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

/**
 * {@link ThreadDeadlockHealthCheck} provider.
 *
 * @since 2.8
 */
@Component
@Qualifier("Thread Deadlock Detector")
public class DeadlockHealthCheckProvider
    implements FactoryBean<HealthCheck>
{
  public HealthCheck getObject() {
    return new ThreadDeadlockHealthCheck();
  }

  public Class<?> getObjectType() {
    return HealthCheck.class;
  }
}
