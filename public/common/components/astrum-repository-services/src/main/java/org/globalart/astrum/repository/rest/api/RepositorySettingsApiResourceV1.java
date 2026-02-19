
package org.globalart.astrum.repository.rest.api;

import java.util.List;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Path;

import static org.globalart.astrum.repository.rest.api.RepositorySettingsApiResourceV1.RESOURCE_URI;
import static org.globalart.astrum.rest.APIConstants.V1_API_PREFIX;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * @since 3.26
 */
@Component
@Path(RESOURCE_URI)
public class RepositorySettingsApiResourceV1
    extends RepositorySettingsApiResource
{
  public static final String RESOURCE_URI = V1_API_PREFIX + "/repositorySettings";

  @Autowired
  public RepositorySettingsApiResourceV1(
      final AuthorizingRepositoryManager authorizingRepositoryManager,
      @Qualifier("default") final ApiRepositoryAdapter defaultAdapter,
      final List<ApiRepositoryAdapter> convertersByFormat)
  {
    super(authorizingRepositoryManager, defaultAdapter, convertersByFormat);
  }
}
