
package org.globalart.astrum.repository.rest.api;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Path;

import static org.globalart.astrum.repository.rest.api.RepositoriesApiResourceV1.RESOURCE_URI;
import static org.globalart.astrum.rest.APIConstants.V1_API_PREFIX;
import org.springframework.stereotype.Component;

/**
 * @since 3.26
 */
@Component
@Path(RESOURCE_URI)
public class RepositoriesApiResourceV1
    extends RepositoriesApiResource
{
  public static final String RESOURCE_URI = V1_API_PREFIX + "/repositories";

  @Autowired
  public RepositoriesApiResourceV1(final AuthorizingRepositoryManager authorizingRepositoryManager) {
    super(authorizingRepositoryManager);
  }
}
