
package org.globalart.astrum.security.internal.rest;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Path;

import org.globalart.astrum.security.SecuritySystem;

import static org.globalart.astrum.security.internal.rest.UserApiResourceV1.RESOURCE_URI;
import org.springframework.stereotype.Component;

/**
 * @since 3.26
 */
@Component
@Path(RESOURCE_URI)
public class UserApiResourceV1
    extends UserApiResource
{
  static final String RESOURCE_URI = SecurityApiConstants.V1_RESOURCE_URI + "users/";

  @Autowired
  public UserApiResourceV1(
      final SecuritySystem securitySystem)
  {
    super(securitySystem);
  }
}
