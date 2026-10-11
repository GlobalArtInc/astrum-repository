
package org.globalart.astrum.coreui.internal.roles;

import java.util.List;
import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Consumes;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.Produces;

import org.globalart.astrum.rest.Resource;
import org.globalart.astrum.security.authz.AuthorizationManager;

import org.apache.shiro.authz.annotation.RequiresAuthentication;
import org.apache.shiro.authz.annotation.RequiresPermissions;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.util.stream.Collectors.toList;
import static jakarta.ws.rs.core.MediaType.APPLICATION_JSON;
import static org.globalart.astrum.security.user.UserManager.DEFAULT_SOURCE;

import org.springframework.stereotype.Component;

/**
 * REST resource for role-related UI operations.
 */
@Component
@Consumes(APPLICATION_JSON)
@Produces(APPLICATION_JSON)
@Path("/internal/ui/roles")
public class RolesUIResource
    implements Resource
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final List<AuthorizationManager> authorizationManagers;

  @Autowired
  public RolesUIResource(final List<AuthorizationManager> authorizationManagers) {
    this.authorizationManagers = checkNotNull(authorizationManagers);
  }

  /**
   * Retrieves available role sources (excluding the default source).
   *
   * @return list of role sources
   */
  @RequiresAuthentication
  @RequiresPermissions("astrum:roles:read")
  @GET
  @Path("/sources")
  public List<RoleSourceUIResponse> listRoleSources() {
    return authorizationManagers.stream()
        .filter(manager -> !DEFAULT_SOURCE.equals(manager.getSource()))
        .map(manager -> new RoleSourceUIResponse(manager.getSource(), manager.getSource()))
        .collect(toList());
  }
}
