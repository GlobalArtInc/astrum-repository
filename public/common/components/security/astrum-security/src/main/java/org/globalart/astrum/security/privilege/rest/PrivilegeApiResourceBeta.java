
package org.globalart.astrum.security.privilege.rest;

import java.util.List;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Path;

import org.globalart.astrum.rest.APIConstants;
import org.globalart.astrum.security.SecurityHelper;
import org.globalart.astrum.security.SecuritySystem;
import org.globalart.astrum.security.internal.rest.SecurityApiConstants;
import org.globalart.astrum.security.privilege.PrivilegeDescriptor;

import io.swagger.v3.oas.annotations.Hidden;

import static org.globalart.astrum.security.privilege.rest.PrivilegeApiResourceBeta.RESOURCE_URI;
import org.springframework.stereotype.Component;

/**
 * @since 3.26
 * @deprecated beta prefix is being phased out, prefer starting new APIs with {@link APIConstants#V1_API_PREFIX} instead
 */
@Hidden
@Component
@Path(RESOURCE_URI)
@Deprecated
public class PrivilegeApiResourceBeta
    extends PrivilegeApiResource
{
  static final String RESOURCE_URI = SecurityApiConstants.BETA_RESOURCE_URI + "privileges";

  @Autowired
  public PrivilegeApiResourceBeta(
      final SecuritySystem securitySystem,
      final SecurityHelper securityHelper,
      final List<PrivilegeDescriptor> privilegeDescriptorsList)
  {
    super(securitySystem, securityHelper, privilegeDescriptorsList);
  }
}
