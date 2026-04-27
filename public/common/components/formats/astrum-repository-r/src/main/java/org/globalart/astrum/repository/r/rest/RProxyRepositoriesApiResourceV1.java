
package org.globalart.astrum.repository.r.rest;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Path;

import org.globalart.astrum.repository.rest.api.RepositoriesApiResourceV1;

import static org.globalart.astrum.repository.r.rest.RProxyRepositoriesApiResourceV1.RESOURCE_URI;
import org.springframework.stereotype.Component;

@Component
@Path(RESOURCE_URI)
public class RProxyRepositoriesApiResourceV1
    extends RProxyRepositoriesApiResource
{
  static final String RESOURCE_URI = RepositoriesApiResourceV1.RESOURCE_URI + "/r/proxy";

  @Autowired
  public void setConfigurationConverter(
      final RProxyRepositoryApiRequestToConfigurationConverter configurationConverter)
  {
    super.setConfigurationConverter(configurationConverter);
  }
}
