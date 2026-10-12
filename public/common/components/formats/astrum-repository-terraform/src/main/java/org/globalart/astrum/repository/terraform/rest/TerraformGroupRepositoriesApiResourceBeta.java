package org.globalart.astrum.repository.terraform.rest;

import jakarta.ws.rs.Path;

import org.globalart.astrum.repository.rest.api.RepositoriesApiResourceBeta;

import io.swagger.v3.oas.annotations.Hidden;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.terraform.rest.TerraformGroupRepositoriesApiResourceBeta.RESOURCE_URI;

@Hidden
@Component
@Path(RESOURCE_URI)
@Deprecated
public class TerraformGroupRepositoriesApiResourceBeta
    extends TerraformGroupRepositoriesApiResource
{
  static final String RESOURCE_URI = RepositoriesApiResourceBeta.RESOURCE_URI + "/terraform/group";

  @Autowired
  public void setConfigurationConverter(
      final TerraformGroupRepositoryApiRequestToConfigurationConverter configurationConverter)
  {
    super.setConfigurationConverter(configurationConverter);
  }
}
