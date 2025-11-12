
package org.globalart.astrum.internal.metrics;

import jakarta.ws.rs.GET;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.Produces;
import jakarta.ws.rs.core.CacheControl;
import jakarta.ws.rs.core.MediaType;
import jakarta.ws.rs.core.Response;

import org.globalart.astrum.rest.Resource;

import org.apache.shiro.authz.annotation.RequiresPermissions;
import org.springframework.stereotype.Component;

/**
 * Resource to ping for uptime.
 */
@Path("/metrics/ping")
@Component
public class PingResource
    implements Resource
{
  private static final String CONTENT = "pong";

  private final CacheControl cacheControl = new CacheControl();

  public PingResource() {
    cacheControl.setMustRevalidate(true);
    cacheControl.setNoCache(true);
    cacheControl.setNoStore(true);
  }

  @GET
  @Produces(MediaType.TEXT_PLAIN)
  @RequiresPermissions("astrum:metrics:read")
  public Response ping() {
    return Response.ok(CONTENT, MediaType.TEXT_PLAIN).cacheControl(cacheControl).build();
  }
}
