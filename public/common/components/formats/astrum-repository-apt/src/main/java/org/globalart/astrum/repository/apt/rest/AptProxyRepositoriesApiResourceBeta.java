
package org.globalart.astrum.repository.apt.rest;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Path;

import org.globalart.astrum.repository.rest.api.RepositoriesApiResourceBeta;
import org.globalart.astrum.rest.APIConstants;

import io.swagger.v3.oas.annotations.Hidden;

import static org.globalart.astrum.repository.apt.rest.AptProxyRepositoriesApiResourceBeta.RESOURCE_URI;
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
public class AptProxyRepositoriesApiResourceBeta
    extends AptProxyRepositoriesApiResource
{
  static final String RESOURCE_URI = RepositoriesApiResourceBeta.RESOURCE_URI + "/apt/proxy";

  @Autowired
  public void setConfigurationConverter(
      final AptProxyRepositoryApiRequestToConfigurationConverter configurationConverter)
  {
    super.setConfigurationConverter(configurationConverter);
  }
}
