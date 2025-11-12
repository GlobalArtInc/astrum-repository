
package org.globalart.astrum.internal.metrics;

import java.util.List;
import java.util.Map;

import org.globalart.astrum.common.QualifierUtil;
import org.globalart.astrum.common.app.ManagedLifecycle;
import org.globalart.astrum.common.stateguard.StateGuardLifecycleSupport;
import org.globalart.astrum.systemchecks.ConditionallyAppliedHealthCheck;

import com.codahale.metrics.health.HealthCheckRegistry;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.app.ManagedLifecycle.Phase.SERVICES;

@Component
@ManagedLifecycle(phase = SERVICES)
public class ConditionallyAppliedHealthCheckRegistrar
    extends StateGuardLifecycleSupport
{
  private final HealthCheckRegistry healthCheckRegistry;

  private final Map<String, ConditionallyAppliedHealthCheck> conditionallyAppliedHealthChecks;

  @Autowired
  public ConditionallyAppliedHealthCheckRegistrar(
      final HealthCheckRegistry healthCheckRegistry,
      final List<ConditionallyAppliedHealthCheck> conditionallyAppliedHealthChecksList)
  {
    this.healthCheckRegistry = checkNotNull(healthCheckRegistry);
    this.conditionallyAppliedHealthChecks =
        QualifierUtil.buildQualifierBeanMap(checkNotNull(conditionallyAppliedHealthChecksList));
  }

  @Override
  protected void doStart() throws Exception {
    conditionallyAppliedHealthChecks.forEach((name, check) -> {
      if (check.shouldApply()) {
        healthCheckRegistry.register(name, check);
      }
    });
  }

  @Override
  protected void doStop() throws Exception {
    conditionallyAppliedHealthChecks.forEach((name, check) -> {
      healthCheckRegistry.unregister(name);
    });
  }
}
