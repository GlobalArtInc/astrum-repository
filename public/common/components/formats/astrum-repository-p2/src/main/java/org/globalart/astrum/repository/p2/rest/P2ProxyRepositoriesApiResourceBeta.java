package org.globalart.astrum.repository.p2.rest;

import org.globalart.astrum.repository.rest.api.RepositoriesApiResourceBeta;
import org.globalart.astrum.rest.APIConstants;

import io.swagger.v3.oas.annotations.Hidden;
import jakarta.ws.rs.Path;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.p2.rest.P2ProxyRepositoriesApiResourceBeta.RESOURCE_URI;

@Hidden
@Component
@Path(RESOURCE_URI)
@Deprecated
public class P2ProxyRepositoriesApiResourceBeta
    extends P2ProxyRepositoriesApiResource
{
  static final String RESOURCE_URI = RepositoriesApiResourceBeta.RESOURCE_URI + "/p2/proxy";

  @Autowired
  public void setConfigurationConverter(
      final P2ProxyRepositoryApiRequestToConfigurationConverter configurationConverter)
  {
    super.setConfigurationConverter(configurationConverter);
  }
}
