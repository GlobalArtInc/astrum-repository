
package org.globalart.astrum.repository.rest.api;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Path;

import org.globalart.astrum.rest.APIConstants;

import io.swagger.v3.oas.annotations.Hidden;

import static org.globalart.astrum.repository.rest.api.RepositoriesApiResourceBeta.RESOURCE_URI;
import static org.globalart.astrum.rest.APIConstants.BETA_API_PREFIX;
import org.springframework.stereotype.Component;

/**
 * @since 3.26
 * @deprecated the 'beta' prefix is being phased out, prefer starting new APIs with {@link APIConstants#V1_API_PREFIX}
 *             instead. Support backward compatibility.
 */
@Hidden
@Component
@Path(RESOURCE_URI)
@Deprecated
public class RepositoriesApiResourceBeta
    extends RepositoriesApiResource
{
  /**
   * @deprecated the 'beta' prefix is being phased out,
   *             prefer starting new APIs with {@link APIConstants#V1_API_PREFIX} instead.
   */
  @Deprecated
  public static final String RESOURCE_URI = BETA_API_PREFIX + "/repositories";

  @Autowired
  public RepositoriesApiResourceBeta(final AuthorizingRepositoryManager authorizingRepositoryManager) {
    super(authorizingRepositoryManager);
  }
}
