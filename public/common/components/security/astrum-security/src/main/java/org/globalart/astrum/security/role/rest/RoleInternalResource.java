
package org.globalart.astrum.security.role.rest;

import java.util.Comparator;
import java.util.List;
import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Consumes;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.Produces;
import jakarta.ws.rs.QueryParam;
import jakarta.ws.rs.core.Response.Status;

import org.globalart.astrum.rest.Resource;
import org.globalart.astrum.rest.WebApplicationMessageException;
import org.globalart.astrum.security.SecuritySystem;
import org.globalart.astrum.security.authz.NoSuchAuthorizationManagerException;

import org.apache.commons.lang3.StringUtils;
import org.apache.shiro.authz.annotation.RequiresAuthentication;
import org.apache.shiro.authz.annotation.RequiresPermissions;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.util.stream.Collectors.toList;
import static jakarta.ws.rs.core.MediaType.APPLICATION_JSON;
import static org.globalart.astrum.security.role.rest.RoleApiResource.SOURCE_NOT_FOUND;
import org.springframework.stereotype.Component;

@Component
@Consumes(APPLICATION_JSON)
@Produces(APPLICATION_JSON)
@Path(RoleInternalResource.RESOURCE_PATH)
public class RoleInternalResource
    implements Resource
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  static final String RESOURCE_PATH = "internal/ui/roles";

  private final SecuritySystem securitySystem;

  @Autowired
  public RoleInternalResource(final SecuritySystem securitySystem) {
    this.securitySystem = checkNotNull(securitySystem);
  }

  @GET
  @RequiresAuthentication
  @RequiresPermissions("astrum:roles:read")
  public List<RoleXOResponse> searchRoles(
      @QueryParam("source") final String source,
      @QueryParam("search") final String search)
  {
    if (StringUtils.isEmpty(source)) {
      return securitySystem.listRoles()
          .stream()
          .map(RoleXOResponse::fromRole)
          .sorted(Comparator.comparing(RoleXOResponse::getId))
          .collect(toList());
    }
    try {
      return securitySystem.searchRoles(source, search)
          .stream()
          .map(RoleXOResponse::fromRole)
          .sorted(Comparator.comparing(RoleXOResponse::getId))
          .collect(toList());
    }
    catch (NoSuchAuthorizationManagerException e) {
      throw buildBadSourceException(source);
    }
  }

  private WebApplicationMessageException buildBadSourceException(final String source) {
    log.debug("attempt to use invalid source {}", source);
    return new WebApplicationMessageException(Status.NOT_FOUND, String.format(SOURCE_NOT_FOUND, source),
        APPLICATION_JSON);
  }
}
