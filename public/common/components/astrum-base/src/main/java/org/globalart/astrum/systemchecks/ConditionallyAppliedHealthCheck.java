
package org.globalart.astrum.systemchecks;

import com.codahale.metrics.health.HealthCheck;

public abstract class ConditionallyAppliedHealthCheck
    extends HealthCheck
{
  abstract public boolean shouldApply();
}
