
package org.globalart.astrum.siesta;

import java.util.List;

import jakarta.ws.rs.Consumes;
import jakarta.ws.rs.DELETE;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.NotFoundException;
import jakarta.ws.rs.PUT;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.PathParam;
import jakarta.ws.rs.Produces;

import org.globalart.astrum.rest.Resource;

import com.google.common.collect.Lists;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static jakarta.ws.rs.core.MediaType.APPLICATION_JSON;
import static jakarta.ws.rs.core.MediaType.APPLICATION_XML;
import org.springframework.stereotype.Component;

@Component
@Path("/user")
public class UserResource
    implements Resource
{
  private final Logger log = LoggerFactory.getLogger(getClass());

  @GET
  @Produces({APPLICATION_XML, APPLICATION_JSON})
  public List<UserXO> get() {
    return Lists.newArrayList(
        new UserXO().withName("foo"),
        new UserXO().withName("bar"));
  }

  @GET
  @Path("/{id}")
  @Produces({APPLICATION_XML, APPLICATION_JSON})
  public UserXO get(@PathParam("id") String id) {
    if ("foo".equals(id)) {
      return new UserXO().withName("foo");
    }
    throw new NotFoundException("User with id '" + id + "' not found");
  }

  @PUT
  @Consumes({APPLICATION_XML, APPLICATION_JSON})
  @Produces({APPLICATION_XML, APPLICATION_JSON})
  public UserXO put(final UserXO user) {
    log.info("PUT user: {}", user);
    return user;
  }

  @DELETE
  @Path("/{id}")
  public void delete(@PathParam("id") String id) {
    if (!"foo".equals(id)) {
      throw new NotFoundException("User with id '" + id + "' not found");
    }
  }
}
