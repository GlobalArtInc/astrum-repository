
package org.globalart.astrum.siesta;

import jakarta.ws.rs.BadRequestException;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.NotFoundException;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.Produces;
import jakarta.ws.rs.WebApplicationException;

import org.globalart.astrum.rest.Resource;

import static jakarta.ws.rs.core.MediaType.APPLICATION_JSON;
import static jakarta.ws.rs.core.MediaType.APPLICATION_XML;
import org.springframework.stereotype.Component;

@Component
@Path("/errors")
public class ErrorsResource
    implements Resource
{
  @GET
  @Path("/NotFoundException")
  @Produces({APPLICATION_XML, APPLICATION_JSON})
  public Object throwObjectNotFoundException() {
    throw new NotFoundException("NotFoundException");
  }

  @GET
  @Path("/BadRequestException")
  @Produces({APPLICATION_XML, APPLICATION_JSON})
  public Object throwBadRequestException() {
    throw new BadRequestException("BadRequestException");
  }

  @GET
  @Path("/406")
  @Produces({APPLICATION_XML, APPLICATION_JSON})
  public Object throw406() {
    throw new WebApplicationException(406);
  }
}
