
package org.globalart.astrum.security.role.rest;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Path;

import org.globalart.astrum.security.SecuritySystem;
import org.globalart.astrum.security.internal.rest.SecurityApiConstants;
import org.globalart.astrum.security.role.RoleAssignabilityChecker;

import org.springframework.stereotype.Component;

/**
 * @since 3.26
 */
@Component
@Path(RoleApiResourceV1.RESOURCE_URI)
public class RoleApiResourceV1
    extends RoleApiResource
{
  public static final String RESOURCE_URI = SecurityApiConstants.V1_RESOURCE_URI + "roles";

  @Autowired
  public RoleApiResourceV1(
      final SecuritySystem securitySystem,
      final RoleAssignabilityChecker roleAssignabilityChecker)
  {
    super(securitySystem, roleAssignabilityChecker);
  }
}
