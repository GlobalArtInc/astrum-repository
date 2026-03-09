
package org.globalart.astrum.script.plugin.internal.rest;

import java.util.List;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Path;

import org.globalart.astrum.rest.APIConstants;
import org.globalart.astrum.security.SecurityHelper;
import org.globalart.astrum.security.SecuritySystem;
import org.globalart.astrum.security.internal.rest.SecurityApiConstants;
import org.globalart.astrum.security.privilege.PrivilegeDescriptor;

import io.swagger.v3.oas.annotations.Hidden;
import org.springframework.stereotype.Component;

/**
 * @since 3.26
 * @deprecated beta prefix is being phased out, prefer starting new APIs with {@link APIConstants#V1_API_PREFIX} instead
 */
@Hidden
@Component
@Path(ScriptPrivilegeApiResourceBeta.RESOURCE_URI)
@Deprecated
public class ScriptPrivilegeApiResourceBeta
    extends ScriptPrivilegeApiResource
{
  static final String RESOURCE_URI = SecurityApiConstants.BETA_RESOURCE_URI + "privileges";

  @Autowired
  public ScriptPrivilegeApiResourceBeta(
      final SecuritySystem securitySystem,
      final SecurityHelper securityHelper,
      final List<PrivilegeDescriptor> privilegeDescriptors)
  {
    super(securitySystem, securityHelper, privilegeDescriptors);
  }
}
