
package org.globalart.astrum.api.rest.common.status;

import jakarta.ws.rs.Consumes;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.Produces;
import jakarta.ws.rs.core.Response;

import org.globalart.astrum.rest.Resource;

import com.codahale.metrics.annotation.Timed;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static jakarta.ws.rs.core.MediaType.APPLICATION_JSON;
import static jakarta.ws.rs.core.Response.ok;
import static org.globalart.astrum.rest.APIConstants.V1_API_PREFIX;
import org.springframework.stereotype.Component;

/**
 * @since 3.15
 */
@Component
@Path(StatusResource.RESOURCE_URI)
@Produces(APPLICATION_JSON)
@Consumes(APPLICATION_JSON)
public class StatusResource
    implements Resource, StatusResourceDoc
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  public static final String RESOURCE_URI = V1_API_PREFIX + "/status";

  @GET
  @Timed
  @Override
  public Response isAvailable() {
    return ok().build();
  }

  @GET
  @Path("/writable")
  @Timed
  @Override
  public Response isWritable() {
    return ok().build();
  }
}
