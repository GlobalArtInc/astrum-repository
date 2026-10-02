
package org.globalart.astrum.api.rest.selfhosted.metrics;

import java.util.Optional;
import java.util.concurrent.TimeUnit;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.QueryParam;
import jakarta.ws.rs.core.CacheControl;
import jakarta.ws.rs.core.MediaType;
import jakarta.ws.rs.core.Response;
import jakarta.ws.rs.core.Response.ResponseBuilder;
import jakarta.ws.rs.core.StreamingOutput;

import org.globalart.astrum.rest.Resource;

import com.codahale.metrics.MetricFilter;
import com.codahale.metrics.MetricRegistry;
import com.codahale.metrics.json.MetricsModule;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.ObjectWriter;
import com.fasterxml.jackson.databind.json.JsonMapper;
import org.apache.shiro.authz.annotation.RequiresPermissions;
import org.springframework.stereotype.Component;

import static jakarta.ws.rs.core.HttpHeaders.CONTENT_DISPOSITION;

/**
 * Proivdes access to the metrics registry
 */
@Path("/metrics/data")
@Component
public class MetricsResource
    implements Resource
{
  private final CacheControl cacheControl = new CacheControl();

  private final MetricRegistry registry;

  private final ObjectMapper mapper;

  @Autowired
  public MetricsResource(final MetricRegistry registry, final JsonMapper mapper) {
    this.registry = registry;

    cacheControl.setMustRevalidate(true);
    cacheControl.setNoCache(true);
    cacheControl.setNoStore(true);

    this.mapper = mapper.copy()
        .registerModule(new MetricsModule(TimeUnit.SECONDS, TimeUnit.SECONDS, false, MetricFilter.ALL));
  }

  @GET
  @RequiresPermissions("astrum:metrics:read")
  public Response get(@QueryParam("download") final Boolean download, @QueryParam("pretty") final Boolean pretty) {

    ResponseBuilder response = Response.ok(output(pretty), MediaType.APPLICATION_JSON).cacheControl(cacheControl);

    Optional.ofNullable(download)
        // hacky we only care if download is true
        .filter(val -> val)
        .ifPresent(val -> response.header(CONTENT_DISPOSITION, "attachment; filename='metrics.json'"));

    return response.build();
  }

  private ObjectWriter writer(final Boolean pretty) {
    if (Boolean.TRUE.equals(pretty)) {
      return mapper.writerWithDefaultPrettyPrinter();
    }
    return mapper.writer();
  }

  private StreamingOutput output(final Boolean pretty) {
    return out -> writer(pretty).writeValue(out, registry);
  }
}
