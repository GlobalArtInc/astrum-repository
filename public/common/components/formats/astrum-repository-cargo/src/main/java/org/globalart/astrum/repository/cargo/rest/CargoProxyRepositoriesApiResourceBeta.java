
package org.globalart.astrum.repository.cargo.rest;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Path;

import org.globalart.astrum.repository.rest.api.RepositoriesApiResourceBeta;

import io.swagger.v3.oas.annotations.Hidden;

import static org.globalart.astrum.repository.cargo.rest.CargoProxyRepositoriesApiResourceBeta.RESOURCE_URI;
import org.springframework.stereotype.Component;

@Hidden
@Component
@Path(RESOURCE_URI)
@Deprecated
public class CargoProxyRepositoriesApiResourceBeta
    extends CargoProxyRepositoriesApiResource
{
  static final String RESOURCE_URI = RepositoriesApiResourceBeta.RESOURCE_URI + "/cargo/proxy";

  @Autowired
  public void setConfigurationConverter(
      final CargoProxyRepositoryApiRequestToConfigurationConverter configurationConverter)
  {
    super.setConfigurationConverter(configurationConverter);
  }
}
