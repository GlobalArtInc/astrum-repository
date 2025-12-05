
package org.globalart.astrum.internal.atlas.customizers;

import java.io.File;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.UncheckedIOException;
import java.lang.management.ManagementFactory;
import java.util.SortedMap;
import java.util.concurrent.TimeUnit;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.supportzip.GeneratedContentSourceSupport;
import org.globalart.astrum.supportzip.SupportBundle;
import org.globalart.astrum.supportzip.SupportBundleCustomizer;

import com.codahale.metrics.MetricRegistry;
import com.codahale.metrics.health.HealthCheck.Result;
import com.codahale.metrics.health.HealthCheckRegistry;
import com.codahale.metrics.json.HealthCheckModule;
import com.codahale.metrics.json.MetricsModule;
import com.codahale.metrics.jvm.ThreadDump;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.supportzip.SupportBundle.ContentSource.Priority.HIGH;
import static org.globalart.astrum.supportzip.SupportBundle.ContentSource.Priority.OPTIONAL;
import static org.globalart.astrum.supportzip.SupportBundle.ContentSource.Type.METRICS;
import static org.globalart.astrum.supportzip.SupportBundle.ContentSource.Type.SYSINFO;
import static org.globalart.astrum.supportzip.SupportBundle.ContentSource.Type.THREAD;
import org.springframework.stereotype.Component;

/**
 * Adds metrics (threads,metrics,healthcheck) to support bundle.
 *
 * @since 2.7
 */
@Component
public class MetricsCustomizer
    implements SupportBundleCustomizer
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final MetricRegistry metricRegistry;

  private final HealthCheckRegistry healthCheckRegistry;

  private final ThreadDump threadDump;

  private final ObjectMapper healthCheckObjectMapper;

  private final ObjectMapper metricsObjectMapper;

  @Autowired
  public MetricsCustomizer(final MetricRegistry metricRegistry, final HealthCheckRegistry healthCheckRegistry) {
    this.metricRegistry = checkNotNull(metricRegistry);
    this.healthCheckRegistry = checkNotNull(healthCheckRegistry);
    this.threadDump = new ThreadDump(ManagementFactory.getThreadMXBean());
    this.healthCheckObjectMapper = new ObjectMapper().registerModule(new HealthCheckModule());
    this.metricsObjectMapper = new ObjectMapper().registerModule(new MetricsModule(
        TimeUnit.SECONDS, // rate-unit
        TimeUnit.SECONDS, // duration-unit
        false // show-samples
    ));
  }

  @Override
  public void customize(final SupportBundle supportBundle) {
    // add thread-dump
    supportBundle.add(new GeneratedContentSourceSupport(THREAD, "info/threads.txt", HIGH)
    {
      @Override
      protected void generate(final File file) {
        try (FileOutputStream fos = new FileOutputStream(file)) {
          threadDump.dump(fos);
        }
        catch (IOException e) {
          throw new UncheckedIOException(e);
        }
      }
    });

    // add healthchecks
    supportBundle.add(new GeneratedContentSourceSupport(SYSINFO, "info/healthcheck.json", OPTIONAL)
    {
      @Override
      protected void generate(final File file) {
        SortedMap<String, Result> results = healthCheckRegistry.runHealthChecks();
        try (FileOutputStream fos = new FileOutputStream(file)) {
          healthCheckObjectMapper.writerWithDefaultPrettyPrinter().writeValue(fos, results);
        }
        catch (IOException e) {
          throw new UncheckedIOException(e);
        }
      }
    });

    // add metrics
    supportBundle.add(new GeneratedContentSourceSupport(METRICS, "info/metrics.json", OPTIONAL)
    {
      @Override
      protected void generate(final File file) {
        try (FileOutputStream fos = new FileOutputStream(file)) {
          metricsObjectMapper.writerWithDefaultPrettyPrinter().writeValue(fos, metricRegistry);
        }
        catch (IOException e) {
          throw new UncheckedIOException(e);
        }
      }
    });
  }
}
