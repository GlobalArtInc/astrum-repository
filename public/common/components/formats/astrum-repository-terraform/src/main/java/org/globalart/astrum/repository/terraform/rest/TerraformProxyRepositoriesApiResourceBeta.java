package org.globalart.astrum.repository.terraform.rest;

import jakarta.ws.rs.Path;

import org.globalart.astrum.repository.rest.api.RepositoriesApiResourceBeta;

import io.swagger.v3.oas.annotations.Hidden;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.terraform.rest.TerraformProxyRepositoriesApiResourceBeta.RESOURCE_URI;

@Hidden
@Component
@Path(RESOURCE_URI)
@Deprecated
public class TerraformProxyRepositoriesApiResourceBeta
    extends TerraformProxyRepositoriesApiResource
{
  static final String RESOURCE_URI = RepositoriesApiResourceBeta.RESOURCE_URI + "/terraform/proxy";

  @Autowired
  public void setConfigurationConverter(
      final TerraformProxyRepositoryApiRequestToConfigurationConverter configurationConverter)
  {
    super.setConfigurationConverter(configurationConverter);
  }
}
