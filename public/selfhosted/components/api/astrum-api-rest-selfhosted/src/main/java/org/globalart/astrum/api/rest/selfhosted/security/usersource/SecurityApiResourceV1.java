
package org.globalart.astrum.api.rest.selfhosted.security.usersource;

import java.util.List;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Path;

import org.globalart.astrum.security.internal.rest.SecurityApiConstants;
import org.globalart.astrum.security.user.UserManager;

import org.springframework.stereotype.Component;

/**
 * @since 3.26
 */
@Component
@Path(SecurityApiConstants.V1_RESOURCE_URI)
public class SecurityApiResourceV1
    extends SecurityApiResource
{
  @Autowired
  public SecurityApiResourceV1(final List<UserManager> userManagersList) {
    super(userManagersList);
  }
}
