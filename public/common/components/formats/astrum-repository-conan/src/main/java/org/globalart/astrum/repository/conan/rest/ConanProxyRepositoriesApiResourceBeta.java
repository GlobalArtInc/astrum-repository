package org.globalart.astrum.repository.conan.rest;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Path;

import org.globalart.astrum.repository.rest.api.RepositoriesApiResourceBeta;

import io.swagger.v3.oas.annotations.Hidden;

import static org.globalart.astrum.repository.conan.rest.ConanProxyRepositoriesApiResourceBeta.RESOURCE_URI;
import org.springframework.stereotype.Component;

@Hidden
@Component
@Path(RESOURCE_URI)
@Deprecated
public class ConanProxyRepositoriesApiResourceBeta
    extends ConanProxyRepositoriesApiResource
{
  static final String RESOURCE_URI = RepositoriesApiResourceBeta.RESOURCE_URI + "/conan/proxy";

  @Autowired
  public void setConfigurationConverter(
      final ConanProxyRepositoryApiRequestToConfigurationConverter configurationConverter)
  {
    super.setConfigurationConverter(configurationConverter);
  }
}
