
package org.globalart.astrum.api.rest.selfhosted.status;

import java.util.SortedMap;

import jakarta.ws.rs.GET;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.Produces;

import org.globalart.astrum.rest.Resource;

import com.codahale.metrics.health.HealthCheck.Result;
import com.codahale.metrics.health.HealthCheckRegistry;
import org.springframework.beans.factory.annotation.Autowired;
import org.apache.shiro.authz.annotation.RequiresAuthentication;
import org.apache.shiro.authz.annotation.RequiresPermissions;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static jakarta.ws.rs.core.MediaType.APPLICATION_JSON;
import static org.globalart.astrum.rest.APIConstants.INTERNAL_API_PREFIX;

/**
 * @since 3.20
 */
@Component
@Path(HealthCheckResource.RESOURCE_URI)
@Produces(APPLICATION_JSON)
public class HealthCheckResource
    implements Resource
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  public static final String RESOURCE_URI = INTERNAL_API_PREFIX + "/ui/status-check";

  private HealthCheckRegistry registry;

  @Autowired
  public HealthCheckResource(final HealthCheckRegistry registry) {
    this.registry = checkNotNull(registry);
  }

  @GET
  @RequiresAuthentication
  @RequiresPermissions("astrum:metrics:read")
  public SortedMap<String, Result> getSystemStatusChecks() {
    return registry.runHealthChecks();
  }
}
