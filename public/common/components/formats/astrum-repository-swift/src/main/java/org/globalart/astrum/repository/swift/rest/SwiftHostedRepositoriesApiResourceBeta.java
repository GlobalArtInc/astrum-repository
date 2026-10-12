package org.globalart.astrum.repository.swift.rest;

import jakarta.ws.rs.Path;

import org.globalart.astrum.repository.rest.api.RepositoriesApiResourceBeta;

import io.swagger.v3.oas.annotations.Hidden;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.swift.rest.SwiftHostedRepositoriesApiResourceBeta.RESOURCE_URI;

@Hidden
@Component
@Path(RESOURCE_URI)
@Deprecated
public class SwiftHostedRepositoriesApiResourceBeta
    extends SwiftHostedRepositoriesApiResource
{
  static final String RESOURCE_URI = RepositoriesApiResourceBeta.RESOURCE_URI + "/swift/hosted";

  @Autowired
  public void setConfigurationConverter(
      final SwiftHostedRepositoryApiRequestToConfigurationConverter configurationConverter)
  {
    super.setConfigurationConverter(configurationConverter);
  }
}
