
package org.globalart.astrum.api.rest.selfhosted.user;

import jakarta.ws.rs.Path;

import org.globalart.astrum.security.SecuritySystem;
import org.globalart.astrum.security.config.AdminPasswordFileManager;
import org.globalart.astrum.security.internal.rest.SecurityApiConstants;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

/**
 * @since 3.26
 */
@Component
@Path(UserApiResourceV1.RESOURCE_URI)
public class UserApiResourceV1
    extends UserApiResource
{
  static final String RESOURCE_URI = SecurityApiConstants.V1_RESOURCE_URI + "users/";

  @Autowired
  public UserApiResourceV1(
      final SecuritySystem securitySystem,
      final AdminPasswordFileManager adminPasswordFileManager)
  {
    super(securitySystem, adminPasswordFileManager);
  }
}
