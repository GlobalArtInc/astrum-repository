
package org.globalart.astrum.repository.npm.rest;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Path;

import org.globalart.astrum.repository.rest.api.RepositoriesApiResourceBeta;
import org.globalart.astrum.rest.APIConstants;

import io.swagger.v3.oas.annotations.Hidden;

import static org.globalart.astrum.repository.npm.rest.NpmGroupRepositoriesApiResourceBeta.RESOURCE_URI;
import org.springframework.stereotype.Component;

/**
 * @since 3.26
 * @deprecated the 'beta' prefix is being phased out, prefer starting new APIs with {@link APIConstants#V1_API_PREFIX}
 *             instead. Support backward compatibility.
 */
@Hidden
@Component
@Path(RESOURCE_URI)
@Deprecated
public class NpmGroupRepositoriesApiResourceBeta
    extends NpmGroupRepositoriesApiResource
{
  static final String RESOURCE_URI = RepositoriesApiResourceBeta.RESOURCE_URI + "/npm/group";

  @Autowired
  public void setConfigurationConverter(
      final NpmGroupRepositoryApiRequestToConfigurationConverter configurationConverter)
  {
    super.setConfigurationConverter(configurationConverter);
  }
}
