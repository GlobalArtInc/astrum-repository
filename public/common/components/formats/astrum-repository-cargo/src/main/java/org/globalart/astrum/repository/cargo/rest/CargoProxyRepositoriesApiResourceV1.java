
package org.globalart.astrum.repository.cargo.rest;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Path;

import org.globalart.astrum.repository.rest.api.RepositoriesApiResourceV1;

import static org.globalart.astrum.repository.cargo.rest.CargoProxyRepositoriesApiResourceV1.RESOURCE_URI;
import org.springframework.stereotype.Component;

@Component
@Path(RESOURCE_URI)
public class CargoProxyRepositoriesApiResourceV1
    extends CargoProxyRepositoriesApiResource
{
  static final String RESOURCE_URI = RepositoriesApiResourceV1.RESOURCE_URI + "/cargo/proxy";

  @Autowired
  public void setConfigurationConverter(
      final CargoProxyRepositoryApiRequestToConfigurationConverter configurationConverter)
  {
    super.setConfigurationConverter(configurationConverter);
  }
}
