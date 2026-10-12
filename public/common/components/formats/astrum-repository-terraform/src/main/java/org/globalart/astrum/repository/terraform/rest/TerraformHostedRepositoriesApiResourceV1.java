package org.globalart.astrum.repository.terraform.rest;

import jakarta.ws.rs.Path;

import org.globalart.astrum.repository.rest.api.RepositoriesApiResourceV1;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.terraform.rest.TerraformHostedRepositoriesApiResourceV1.RESOURCE_URI;

@Component
@Path(RESOURCE_URI)
public class TerraformHostedRepositoriesApiResourceV1
    extends TerraformHostedRepositoriesApiResource
{
  static final String RESOURCE_URI = RepositoriesApiResourceV1.RESOURCE_URI + "/terraform/hosted";

  @Autowired
  public void setConfigurationConverter(
      final TerraformHostedRepositoryApiRequestToConfigurationConverter configurationConverter)
  {
    super.setConfigurationConverter(configurationConverter);
  }
}
