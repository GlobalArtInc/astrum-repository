
package org.globalart.astrum.repository.r.rest;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Path;

import org.globalart.astrum.repository.rest.api.RepositoriesApiResourceBeta;

import io.swagger.v3.oas.annotations.Hidden;

import static org.globalart.astrum.repository.r.rest.RHostedRepositoriesApiResourceBeta.RESOURCE_URI;
import org.springframework.stereotype.Component;

@Hidden
@Component
@Path(RESOURCE_URI)
@Deprecated
public class RHostedRepositoriesApiResourceBeta
    extends RHostedRepositoriesApiResource
{
  static final String RESOURCE_URI = RepositoriesApiResourceBeta.RESOURCE_URI + "/r/hosted";

  @Autowired
  public void setConfigurationConverter(
      final RHostedRepositoryApiRequestToConfigurationConverter configurationConverter)
  {
    super.setConfigurationConverter(configurationConverter);
  }
}
