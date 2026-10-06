
package org.globalart.astrum.metrics;

import java.lang.management.ManagementFactory;

import jakarta.annotation.PostConstruct;

import com.codahale.metrics.MetricRegistry;
import com.codahale.metrics.jvm.BufferPoolMetricSet;
import com.codahale.metrics.jvm.FileDescriptorRatioGauge;
import com.codahale.metrics.jvm.GarbageCollectorMetricSet;
import com.codahale.metrics.jvm.JvmAttributeGaugeSet;
import com.codahale.metrics.jvm.MemoryUsageGaugeSet;
import com.codahale.metrics.jvm.ThreadStatesGaugeSet;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.context.annotation.Configuration;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.codahale.metrics.MetricRegistry.name;
import static org.globalart.astrum.common.metrics.MetricsConstants.ASTRUM_METRICS_REGISTRY_NAME;

/**
 * Registers JVM infrastructure metrics (memory, threads, GC, etc.) into the Astrum metric registry.
 */
@Configuration
public class JvmMetricsConfiguration
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final MetricRegistry registry;

  public JvmMetricsConfiguration(@Qualifier(ASTRUM_METRICS_REGISTRY_NAME) final MetricRegistry registry) {
    this.registry = registry;
  }

  @PostConstruct
  void registerJvmMetrics() {
    log.debug("Registering JVM infrastructure metrics");
    registry.register(name("jvm", "vm"), new JvmAttributeGaugeSet());
    registry.register(name("jvm", "memory"), new MemoryUsageGaugeSet());
    registry.register(name("jvm", "buffers"), new BufferPoolMetricSet(ManagementFactory.getPlatformMBeanServer()));
    registry.register(name("jvm", "fd_usage"), new FileDescriptorRatioGauge());
    registry.register(name("jvm", "thread-states"), new ThreadStatesGaugeSet());
    registry.register(name("jvm", "garbage-collectors"), new GarbageCollectorMetricSet());
  }
}
