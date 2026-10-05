
package org.globalart.astrum.api.rest.selfhosted.status;

import java.util.SortedMap;

import jakarta.ws.rs.Consumes;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.Produces;

import org.globalart.astrum.rest.Resource;

import com.codahale.metrics.annotation.Timed;
import com.codahale.metrics.health.HealthCheck.Result;
import com.codahale.metrics.health.HealthCheckRegistry;
import org.springframework.beans.factory.annotation.Autowired;
import org.apache.shiro.authz.annotation.RequiresAuthentication;
import org.apache.shiro.authz.annotation.RequiresPermissions;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static jakarta.ws.rs.core.MediaType.APPLICATION_JSON;
import static org.globalart.astrum.api.rest.selfhosted.status.StatusCheckResource.RESOURCE_PATH;
import static org.globalart.astrum.rest.APIConstants.V1_API_PREFIX;

@Component
@Produces(APPLICATION_JSON)
@Consumes(APPLICATION_JSON)
@Path(RESOURCE_PATH)
public class StatusCheckResource
    implements Resource, StatusCheckResourceDoc
{
  static final String RESOURCE_PATH = V1_API_PREFIX + "/status/check";

  private final HealthCheckRegistry registry;

  @Autowired
  public StatusCheckResource(final HealthCheckRegistry registry) {
    this.registry = checkNotNull(registry);
  }

  @GET
  @Timed
  @RequiresAuthentication
  @RequiresPermissions("astrum:metrics:read")
  @Override
  public SortedMap<String, Result> getSystemStatusChecks() {
    return registry.runHealthChecks();
  }
}
