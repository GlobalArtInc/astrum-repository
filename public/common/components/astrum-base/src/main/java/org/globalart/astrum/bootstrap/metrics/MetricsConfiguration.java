
package org.globalart.astrum.bootstrap.metrics;

import java.util.List;

import org.globalart.astrum.common.QualifierUtil;
import org.globalart.astrum.systemchecks.ConditionallyAppliedHealthCheck;

import com.codahale.metrics.Metric;
import com.codahale.metrics.MetricRegistry;
import com.codahale.metrics.SharedMetricRegistries;
import com.codahale.metrics.health.HealthCheck;
import com.codahale.metrics.health.HealthCheckRegistry;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.context.annotation.Primary;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static org.globalart.astrum.common.metrics.MetricsConstants.ASTRUM_METRICS_REGISTRY_NAME;

@Configuration
public class MetricsConfiguration
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  @Primary
  @Qualifier(ASTRUM_METRICS_REGISTRY_NAME)
  @Bean
  public MetricRegistry astrumMetricRegistry(final List<Metric> metrics) {
    MetricRegistry registry = SharedMetricRegistries.getOrCreate(ASTRUM_METRICS_REGISTRY_NAME);

    for (Metric metric : metrics) {
      // Avoid adding registry to itself
      if (!(metric instanceof MetricRegistry)) {
        log.debug("Registering: {}", metric);
        String name = QualifierUtil.value(metric)
            .orElseGet(metric.getClass()::getName);
        registry.register(name, metric);
      }
    }

    return registry;
  }

  @Bean
  public HealthCheckRegistry healthCheckRegistry(final List<HealthCheck> healthChecks) {
    HealthCheckRegistry registry = new HealthCheckRegistry();

    for (HealthCheck healthCheck : healthChecks) {
      String name = QualifierUtil.value(healthCheck)
          .orElseGet(healthCheck.getClass()::getName);
      if (healthCheck instanceof ConditionallyAppliedHealthCheck) {
        log.debug("Delay Registry of {} Until Conditional Registration", name);
      }
      else {
        log.debug("Registering: {}", healthCheck);
        registry.register(name, healthCheck);
      }
    }

    return registry;
  }
}
