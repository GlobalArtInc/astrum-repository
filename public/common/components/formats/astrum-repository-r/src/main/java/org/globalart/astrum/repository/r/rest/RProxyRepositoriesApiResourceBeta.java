
package org.globalart.astrum.repository.r.rest;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Path;

import org.globalart.astrum.repository.rest.api.RepositoriesApiResourceBeta;

import io.swagger.v3.oas.annotations.Hidden;

import static org.globalart.astrum.repository.r.rest.RProxyRepositoriesApiResourceBeta.RESOURCE_URI;
import org.springframework.stereotype.Component;

@Hidden
@Component
@Path(RESOURCE_URI)
@Deprecated
public class RProxyRepositoriesApiResourceBeta
    extends RProxyRepositoriesApiResource
{
  static final String RESOURCE_URI = RepositoriesApiResourceBeta.RESOURCE_URI + "/r/proxy";

  @Autowired
  public void setConfigurationConverter(
      final RProxyRepositoryApiRequestToConfigurationConverter configurationConverter)
  {
    super.setConfigurationConverter(configurationConverter);
  }
}
