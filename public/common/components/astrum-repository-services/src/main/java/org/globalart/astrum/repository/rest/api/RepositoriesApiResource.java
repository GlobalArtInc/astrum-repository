
package org.globalart.astrum.repository.rest.api;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Consumes;
import jakarta.ws.rs.DELETE;
import jakarta.ws.rs.POST;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.PathParam;
import jakarta.ws.rs.Produces;
import jakarta.ws.rs.core.Response;

import org.globalart.astrum.rest.Resource;
import org.globalart.astrum.rest.WebApplicationMessageException;

import org.apache.shiro.authz.annotation.RequiresAuthentication;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static jakarta.ws.rs.core.MediaType.APPLICATION_JSON;
import static jakarta.ws.rs.core.Response.Status.BAD_REQUEST;
import static jakarta.ws.rs.core.Response.Status.NOT_FOUND;
import static jakarta.ws.rs.core.Response.Status.NO_CONTENT;

/**
 * @since 3.20
 */
@Produces(APPLICATION_JSON)
@Consumes(APPLICATION_JSON)
public class RepositoriesApiResource
    implements Resource, RepositoriesApiResourceDoc
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final AuthorizingRepositoryManager authorizingRepositoryManager;

  @Autowired
  public RepositoriesApiResource(final AuthorizingRepositoryManager authorizingRepositoryManager) {
    this.authorizingRepositoryManager = checkNotNull(authorizingRepositoryManager);
  }

  @Override
  @DELETE
  @Path("/{repositoryName}")
  @RequiresAuthentication
  public Response deleteRepository(@PathParam("repositoryName") final String repositoryName) throws Exception {
    boolean isDeleted = authorizingRepositoryManager.delete(repositoryName);
    return Response.status(isDeleted ? NO_CONTENT : NOT_FOUND).build();
  }

  @POST
  @Path("/{repositoryName}/rebuild-index")
  @RequiresAuthentication
  public void rebuildIndex(@PathParam("repositoryName") final String repositoryName) {
    try {
      authorizingRepositoryManager.rebuildSearchIndex(repositoryName);
    }
    catch (IncompatibleRepositoryException e) {
      log.debug("Not a hosted or proxy repository '{}'", repositoryName, e);
      throw new WebApplicationMessageException(BAD_REQUEST, "\"" + e.getMessage() + "\"", APPLICATION_JSON);
    }
    catch (RepositoryNotFoundException e) {
      log.debug("Repository not found '{}'", repositoryName, e);
      throw new WebApplicationMessageException(NOT_FOUND, "\"" + e.getMessage() + "\"", APPLICATION_JSON);
    }
  }

  @POST
  @Path("/{repositoryName}/invalidate-cache")
  @RequiresAuthentication
  public void invalidateCache(@PathParam("repositoryName") final String repositoryName) {
    try {
      authorizingRepositoryManager.invalidateCache(repositoryName);
    }
    catch (IncompatibleRepositoryException e) {
      log.debug("Not a proxy nor group repository '{}'", repositoryName, e);
      throw new WebApplicationMessageException(BAD_REQUEST, "\"" + e.getMessage() + "\"", APPLICATION_JSON);
    }
    catch (RepositoryNotFoundException e) {
      log.debug("Repository not found '{}'", repositoryName, e);
      throw new WebApplicationMessageException(NOT_FOUND, "\"" + e.getMessage() + "\"", APPLICATION_JSON);
    }
  }
}
