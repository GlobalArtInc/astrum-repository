
package org.globalart.astrum.security.role.rest;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Path;

import org.globalart.astrum.rest.APIConstants;
import org.globalart.astrum.security.SecuritySystem;
import org.globalart.astrum.security.internal.rest.SecurityApiConstants;
import org.globalart.astrum.security.role.RoleAssignabilityChecker;

import io.swagger.v3.oas.annotations.Hidden;
import org.springframework.stereotype.Component;

/**
 * @since 3.26
 * @deprecated beta prefix is being phased out, prefer starting new APIs with {@link APIConstants#V1_API_PREFIX} instead
 */
@Hidden
@Component
@Path(RoleApiResourceBeta.RESOURCE_URI)
@Deprecated
public class RoleApiResourceBeta
    extends RoleApiResource
{
  static final String RESOURCE_URI = SecurityApiConstants.BETA_RESOURCE_URI + "roles";

  @Autowired
  public RoleApiResourceBeta(
      final SecuritySystem securitySystem,
      final RoleAssignabilityChecker roleAssignabilityChecker)
  {
    super(securitySystem, roleAssignabilityChecker);
  }
}
