
package org.globalart.astrum.api.rest.selfhosted.security.usersource;

import java.util.List;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Path;

import org.globalart.astrum.rest.APIConstants;
import org.globalart.astrum.security.internal.rest.SecurityApiConstants;
import org.globalart.astrum.security.user.UserManager;

import io.swagger.v3.oas.annotations.Hidden;

import org.springframework.stereotype.Component;

/**
 * @since 3.26
 * @deprecated beta prefix is being phased out, prefer starting new APIs with {@link APIConstants#V1_API_PREFIX} instead
 */
@Hidden
@Component
@Path(SecurityApiConstants.BETA_RESOURCE_URI)
@Deprecated
public class SecurityApiResourceBeta
    extends SecurityApiResource
{

  @Autowired
  public SecurityApiResourceBeta(final List<UserManager> userManagersList) {
    super(userManagersList);
  }
}
