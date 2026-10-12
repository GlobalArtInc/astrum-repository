package org.globalart.astrum.repository.rubygems.rest;

import jakarta.ws.rs.Path;

import org.globalart.astrum.repository.rest.api.RepositoriesApiResourceV1;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.rubygems.rest.RubygemsGroupRepositoriesApiResourceV1.RESOURCE_URI;

@Component
@Path(RESOURCE_URI)
public class RubygemsGroupRepositoriesApiResourceV1
    extends RubygemsGroupRepositoriesApiResource
{
  static final String RESOURCE_URI = RepositoriesApiResourceV1.RESOURCE_URI + "/rubygems/group";

  @Autowired
  public void setConfigurationConverter(
      final RubygemsGroupRepositoryApiRequestToConfigurationConverter configurationConverter)
  {
    super.setConfigurationConverter(configurationConverter);
  }
}
