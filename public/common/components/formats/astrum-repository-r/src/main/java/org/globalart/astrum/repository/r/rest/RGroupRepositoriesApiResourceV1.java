
package org.globalart.astrum.repository.r.rest;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Path;

import org.globalart.astrum.repository.rest.api.RepositoriesApiResourceV1;

import static org.globalart.astrum.repository.r.rest.RGroupRepositoriesApiResourceV1.RESOURCE_URI;
import org.springframework.stereotype.Component;

@Component
@Path(RESOURCE_URI)
public class RGroupRepositoriesApiResourceV1
    extends RGroupRepositoriesApiResource
{
  static final String RESOURCE_URI = RepositoriesApiResourceV1.RESOURCE_URI + "/r/group";

  @Autowired
  public void setConfigurationConverter(
      final RGroupRepositoryApiRequestToConfigurationConverter configurationConverter)
  {
    super.setConfigurationConverter(configurationConverter);
  }
}
