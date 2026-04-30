package org.globalart.astrum.repository.conan.rest;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Path;

import org.globalart.astrum.repository.rest.api.RepositoriesApiResourceBeta;

import io.swagger.v3.oas.annotations.Hidden;

import static org.globalart.astrum.repository.conan.rest.ConanGroupRepositoriesApiResourceBeta.RESOURCE_URI;
import org.springframework.stereotype.Component;

@Hidden
@Component
@Path(RESOURCE_URI)
@Deprecated
public class ConanGroupRepositoriesApiResourceBeta
    extends ConanGroupRepositoriesApiResource
{
  static final String RESOURCE_URI = RepositoriesApiResourceBeta.RESOURCE_URI + "/conan/group";

  @Autowired
  public void setConfigurationConverter(
      final ConanGroupRepositoryApiRequestToConfigurationConverter configurationConverter)
  {
    super.setConfigurationConverter(configurationConverter);
  }
}
