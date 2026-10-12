package org.globalart.astrum.repository.rubygems.rest;

import jakarta.ws.rs.Path;

import org.globalart.astrum.repository.rest.api.RepositoriesApiResourceV1;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.rubygems.rest.RubygemsHostedRepositoriesApiResourceV1.RESOURCE_URI;

@Component
@Path(RESOURCE_URI)
public class RubygemsHostedRepositoriesApiResourceV1
    extends RubygemsHostedRepositoriesApiResource
{
  static final String RESOURCE_URI = RepositoriesApiResourceV1.RESOURCE_URI + "/rubygems/hosted";

  @Autowired
  public void setConfigurationConverter(
      final RubygemsHostedRepositoryApiRequestToConfigurationConverter configurationConverter)
  {
    super.setConfigurationConverter(configurationConverter);
  }
}
