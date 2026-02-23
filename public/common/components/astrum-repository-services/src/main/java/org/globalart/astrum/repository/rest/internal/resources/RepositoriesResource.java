
package org.globalart.astrum.repository.rest.internal.resources;

import java.util.List;

import org.globalart.astrum.repository.rest.api.RepositoryManagerRESTAdapter;
import org.globalart.astrum.repository.rest.api.RepositoryXO;
import org.globalart.astrum.repository.rest.internal.resources.doc.RepositoriesResourceDoc;
import org.globalart.astrum.rest.Resource;

import jakarta.ws.rs.Consumes;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.PathParam;
import jakarta.ws.rs.Produces;
import org.apache.shiro.authz.annotation.RequiresUser;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static jakarta.ws.rs.core.MediaType.APPLICATION_JSON;
import static org.globalart.astrum.rest.APIConstants.V1_API_PREFIX;

/**
 * @since 3.9
 */
@Component
@Path(RepositoriesResource.RESOURCE_URI)
@Produces(APPLICATION_JSON)
@Consumes(APPLICATION_JSON)
public class RepositoriesResource
    implements Resource, RepositoriesResourceDoc
{
  public static final String RESOURCE_URI = V1_API_PREFIX + "/repositories";

  private final RepositoryManagerRESTAdapter repositoryManagerRESTAdapter;

  @Autowired
  public RepositoriesResource(final RepositoryManagerRESTAdapter repositoryManagerRESTAdapter) {
    this.repositoryManagerRESTAdapter = checkNotNull(repositoryManagerRESTAdapter);
  }

  @RequiresUser
  @GET
  @Override
  public List<RepositoryXO> getRepositories() {
    return repositoryManagerRESTAdapter.getRepositories();
  }

  @RequiresUser
  @GET
  @Override
  @Path("/{repositoryName}")
  public RepositoryXO getRepository(@PathParam("repositoryName") final String repositoryName) {
    return RepositoryXO.fromRepository(
        repositoryManagerRESTAdapter.getRepository(repositoryName),
        repositoryManagerRESTAdapter.getRepositorySize(repositoryName).orElse(null));
  }
}
