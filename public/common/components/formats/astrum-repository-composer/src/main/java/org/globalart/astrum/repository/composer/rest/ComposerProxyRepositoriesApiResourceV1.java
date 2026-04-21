
package org.globalart.astrum.repository.composer.rest;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Path;

import org.globalart.astrum.repository.rest.api.RepositoriesApiResourceV1;

import static org.globalart.astrum.repository.composer.rest.ComposerProxyRepositoriesApiResourceV1.RESOURCE_URI;
import org.springframework.stereotype.Component;

/**
 * @since 3.26
 */
@Component
@Path(RESOURCE_URI)
public class ComposerProxyRepositoriesApiResourceV1
    extends ComposerProxyRepositoriesApiResource
{
  static final String RESOURCE_URI = RepositoriesApiResourceV1.RESOURCE_URI + "/composer/proxy";

  @Autowired
  public void setConfigurationConverter(
      final ComposerProxyRepositoryApiRequestToConfigurationConverter configurationConverter)
  {
    super.setConfigurationConverter(configurationConverter);
  }
}
