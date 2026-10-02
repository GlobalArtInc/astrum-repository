
package org.globalart.astrum.api.rest.selfhosted.metrics;

import java.io.BufferedWriter;
import java.io.OutputStreamWriter;
import java.io.Writer;
import java.util.Set;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.QueryParam;
import jakarta.ws.rs.core.CacheControl;
import jakarta.ws.rs.core.Response;
import jakarta.ws.rs.core.StreamingOutput;

import org.globalart.astrum.rest.Resource;

import com.codahale.metrics.MetricRegistry;
import io.prometheus.client.CollectorRegistry;
import io.prometheus.client.dropwizard.DropwizardExports;
import io.prometheus.client.exporter.common.TextFormat;
import org.apache.shiro.authz.annotation.RequiresPermissions;
import org.springframework.stereotype.Component;

@Path("/metrics/prometheus")
@Component
public class PrometheusMetricsResource
    implements Resource
{
  private final CacheControl cacheControl = new CacheControl();

  @Autowired
  public PrometheusMetricsResource(final MetricRegistry registry) {
    // Export to Prometheus
    new DropwizardExports(registry).register();

    cacheControl.setMustRevalidate(true);
    cacheControl.setNoCache(true);
    cacheControl.setNoStore(true);
  }

  @GET
  @RequiresPermissions("astrum:metrics:read")
  public Response read(@QueryParam("name[]") final Set<String> names) {
    return Response.ok(entity(names), TextFormat.CONTENT_TYPE_004).cacheControl(cacheControl).build();
  }

  private StreamingOutput entity(final Set<String> names) {
    return out -> {
      try (Writer writer = new BufferedWriter(new OutputStreamWriter(out))) {
        TextFormat.write004(writer, CollectorRegistry.defaultRegistry.filteredMetricFamilySamples(names));
      }
    };
  }
}
