
package org.globalart.astrum.internal.status;

import org.slf4j.LoggerFactory;

import com.codahale.metrics.health.HealthCheck;
import com.google.common.base.Preconditions;
import org.slf4j.Logger;

/**
 * Adds component support to all health checks
 *
 * @since 3.16
 */
public abstract class HealthCheckComponentSupport
    extends HealthCheck
{
  protected final Logger log = Preconditions.checkNotNull(this.createLogger());

  protected HealthCheckComponentSupport() {
  }

  protected Logger createLogger() {
    return LoggerFactory.getLogger(getClass());
  }

}
