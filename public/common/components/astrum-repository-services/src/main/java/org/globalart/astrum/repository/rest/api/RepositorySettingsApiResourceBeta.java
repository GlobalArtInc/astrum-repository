
package org.globalart.astrum.repository.rest.api;

import java.util.List;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Path;

import org.globalart.astrum.rest.APIConstants;

import io.swagger.v3.oas.annotations.Hidden;

import static org.globalart.astrum.repository.rest.api.RepositorySettingsApiResourceBeta.RESOURCE_URI;
import static org.globalart.astrum.rest.APIConstants.BETA_API_PREFIX;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * @since 3.26
 * @deprecated the 'beta' prefix is being phased out, prefer starting new APIs with {@link APIConstants#V1_API_PREFIX}
 *             instead. Support backward compatibility.
 */
@Hidden
@Component
@Path(RESOURCE_URI)
@Deprecated
public class RepositorySettingsApiResourceBeta
    extends RepositorySettingsApiResource
{
  public static final String RESOURCE_URI = BETA_API_PREFIX + "/repositories";

  @Autowired
  public RepositorySettingsApiResourceBeta(
      final AuthorizingRepositoryManager authorizingRepositoryManager,
      @Qualifier("default") final ApiRepositoryAdapter defaultAdapter,
      final List<ApiRepositoryAdapter> convertersByFormat)
  {
    super(authorizingRepositoryManager, defaultAdapter, convertersByFormat);
  }
}
