
package org.globalart.astrum.repository.npm.rest;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Path;

import org.globalart.astrum.repository.rest.api.RepositoriesApiResourceV1;

import static org.globalart.astrum.repository.npm.rest.NpmGroupRepositoriesApiResourceV1.RESOURCE_URI;
import org.springframework.stereotype.Component;

/**
 * @since 3.26
 */
@Component
@Path(RESOURCE_URI)
public class NpmGroupRepositoriesApiResourceV1
    extends NpmGroupRepositoriesApiResource
{
  static final String RESOURCE_URI = RepositoriesApiResourceV1.RESOURCE_URI + "/npm/group";

  @Autowired
  public void setConfigurationConverter(
      final NpmGroupRepositoryApiRequestToConfigurationConverter configurationConverter)
  {
    super.setConfigurationConverter(configurationConverter);
  }
}
