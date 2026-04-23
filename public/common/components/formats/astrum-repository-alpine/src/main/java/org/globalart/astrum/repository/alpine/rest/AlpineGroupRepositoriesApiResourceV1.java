
package org.globalart.astrum.repository.alpine.rest;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Path;

import org.globalart.astrum.repository.rest.api.RepositoriesApiResourceV1;

import static org.globalart.astrum.repository.alpine.rest.AlpineGroupRepositoriesApiResourceV1.RESOURCE_URI;
import org.springframework.stereotype.Component;

/**
 * @since 3.26
 */
@Component
@Path(RESOURCE_URI)
public class AlpineGroupRepositoriesApiResourceV1
    extends AlpineGroupRepositoriesApiResource
{
  static final String RESOURCE_URI = RepositoriesApiResourceV1.RESOURCE_URI + "/alpine/group";

  @Autowired
  public void setConfigurationConverter(
      final AlpineGroupRepositoryApiRequestToConfigurationConverter configurationConverter)
  {
    super.setConfigurationConverter(configurationConverter);
  }
}
