package org.globalart.astrum.repository.docker.rest;

import java.io.IOException;
import java.net.ServerSocket;

import org.globalart.astrum.repository.docker.internal.DockerConnectorManager;
import org.globalart.astrum.rest.Resource;

import io.swagger.v3.oas.annotations.Hidden;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.Produces;
import jakarta.ws.rs.QueryParam;
import jakarta.ws.rs.WebApplicationException;
import org.apache.shiro.authz.annotation.RequiresAuthentication;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static jakarta.ws.rs.core.MediaType.APPLICATION_JSON;

@Hidden
@Component
@Produces(APPLICATION_JSON)
@Path(DockerPortSuggestionResource.RESOURCE_PATH)
public class DockerPortSuggestionResource
    implements Resource
{
  static final String RESOURCE_PATH = "internal/ui/docker";

  private static final int FIRST_PORT = 8082;

  private static final int LAST_PORT = 8999;

  private final DockerConnectorManager connectorManager;

  @Autowired
  public DockerPortSuggestionResource(final DockerConnectorManager connectorManager) {
    this.connectorManager = checkNotNull(connectorManager);
  }

  @GET
  @Path("suggest-port")
  @RequiresAuthentication
  public int suggestPort(@QueryParam("exclude") final Integer exclude) {
    for (int port = FIRST_PORT; port <= LAST_PORT; port++) {
      if ((exclude != null && exclude == port) || connectorManager.portOwner(port).isPresent()
          || connectorManager.isApplicationPort(port)) {
        continue;
      }
      if (isFree(port)) {
        return port;
      }
    }
    throw new WebApplicationException("No free port available", 404);
  }

  private static boolean isFree(final int port) {
    try (ServerSocket socket = new ServerSocket(port)) {
      socket.setReuseAddress(true);
      return true;
    }
    catch (IOException e) {
      return false;
    }
  }
}
