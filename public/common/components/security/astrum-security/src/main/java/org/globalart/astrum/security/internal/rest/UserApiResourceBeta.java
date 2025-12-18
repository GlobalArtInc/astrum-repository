
package org.globalart.astrum.security.internal.rest;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Path;

import org.globalart.astrum.rest.APIConstants;
import org.globalart.astrum.security.SecuritySystem;

import io.swagger.v3.oas.annotations.Hidden;

import static org.globalart.astrum.security.internal.rest.UserApiResourceBeta.RESOURCE_URI;
import org.springframework.stereotype.Component;

/**
 * @since 3.26
 * @deprecated beta prefix is being phased out, prefer starting new APIs with {@link APIConstants#V1_API_PREFIX} instead
 */
@Hidden
@Component
@Path(RESOURCE_URI)
@Deprecated
public class UserApiResourceBeta
    extends UserApiResource
{
  static final String RESOURCE_URI = SecurityApiConstants.BETA_RESOURCE_URI + "users/";

  @Autowired
  public UserApiResourceBeta(
      final SecuritySystem securitySystem)
  {
    super(securitySystem);
  }
}
