
package org.globalart.astrum.security.privilege.rest;

import java.util.List;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Path;

import org.globalart.astrum.security.SecurityHelper;
import org.globalart.astrum.security.SecuritySystem;
import org.globalart.astrum.security.internal.rest.SecurityApiConstants;
import org.globalart.astrum.security.privilege.PrivilegeDescriptor;
import org.springframework.stereotype.Component;

/**
 * @since 3.26
 */
@Component
@Path(PrivilegeApiResourceV1.RESOURCE_URI)
public class PrivilegeApiResourceV1
    extends PrivilegeApiResource
{
  static final String RESOURCE_URI = SecurityApiConstants.V1_RESOURCE_URI + "privileges";

  @Autowired
  public PrivilegeApiResourceV1(
      final SecuritySystem securitySystem,
      final SecurityHelper securityHelper,
      final List<PrivilegeDescriptor> privilegeDescriptorsList)
  {
    super(securitySystem, securityHelper, privilegeDescriptorsList);
  }
}
