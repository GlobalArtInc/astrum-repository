package org.globalart.astrum.repository.p2.rest;

import org.globalart.astrum.repository.rest.api.RepositoriesApiResourceV1;

import jakarta.ws.rs.Path;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.p2.rest.P2ProxyRepositoriesApiResourceV1.RESOURCE_URI;

@Component
@Path(RESOURCE_URI)
public class P2ProxyRepositoriesApiResourceV1
    extends P2ProxyRepositoriesApiResource
{
  static final String RESOURCE_URI = RepositoriesApiResourceV1.RESOURCE_URI + "/p2/proxy";

  @Autowired
  public void setConfigurationConverter(
      final P2ProxyRepositoryApiRequestToConfigurationConverter configurationConverter)
  {
    super.setConfigurationConverter(configurationConverter);
  }
}
