package org.globalart.astrum.repository.terraform.rest;

import jakarta.ws.rs.Path;

import org.globalart.astrum.repository.rest.api.RepositoriesApiResourceV1;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.terraform.rest.TerraformProxyRepositoriesApiResourceV1.RESOURCE_URI;

@Component
@Path(RESOURCE_URI)
public class TerraformProxyRepositoriesApiResourceV1
    extends TerraformProxyRepositoriesApiResource
{
  static final String RESOURCE_URI = RepositoriesApiResourceV1.RESOURCE_URI + "/terraform/proxy";

  @Autowired
  public void setConfigurationConverter(
      final TerraformProxyRepositoryApiRequestToConfigurationConverter configurationConverter)
  {
    super.setConfigurationConverter(configurationConverter);
  }
}
