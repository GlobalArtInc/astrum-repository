
package org.globalart.astrum.security.privilege.rest;

import java.util.Comparator;
import java.util.List;
import java.util.Objects;
import java.util.stream.Collectors;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Consumes;
import jakarta.ws.rs.DELETE;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.POST;
import jakarta.ws.rs.PUT;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.PathParam;
import jakarta.ws.rs.Produces;
import jakarta.ws.rs.core.MediaType;
import jakarta.ws.rs.core.Response;
import jakarta.ws.rs.core.Response.Status;

import org.globalart.astrum.rest.Resource;
import org.globalart.astrum.rest.WebApplicationMessageException;
import org.globalart.astrum.security.SecurityHelper;
import org.globalart.astrum.security.SecuritySystem;
import org.globalart.astrum.security.privilege.ApplicationPrivilegeDescriptor;
import org.globalart.astrum.security.privilege.NoSuchPrivilegeException;
import org.globalart.astrum.security.privilege.PrivilegeDescriptor;
import org.globalart.astrum.security.privilege.ReadonlyPrivilegeException;
import org.globalart.astrum.security.privilege.WildcardPrivilegeDescriptor;

import org.apache.shiro.authz.annotation.RequiresAuthentication;
import org.apache.shiro.authz.annotation.RequiresPermissions;

import static jakarta.ws.rs.core.MediaType.APPLICATION_JSON;

/**
 * @since 3.19
 */
@Consumes(APPLICATION_JSON)
@Produces(APPLICATION_JSON)
public class PrivilegeApiResource
    extends PrivilegeApiResourceSupport
    implements Resource, PrivilegeApiResourceDoc
{
  @Autowired
  public PrivilegeApiResource(
      final SecuritySystem securitySystem,
      final SecurityHelper securityHelper,
      final List<PrivilegeDescriptor> privilegeDescriptorsList)
  {
    super(securitySystem, securityHelper, privilegeDescriptorsList);
  }

  @Override
  @GET
  @RequiresAuthentication
  @RequiresPermissions("astrum:privileges:read")
  public List<ApiPrivilege> getPrivileges() {
    return getSecuritySystem().listPrivileges()
        .stream()
        .map(this::toApiPrivilege)
        .filter(Objects::nonNull)
        .sorted(Comparator.comparing(ApiPrivilege::getName))
        .collect(Collectors.toList());
  }

  @Override
  @GET
  @RequiresAuthentication
  @RequiresPermissions("astrum:privileges:read")
  @Path("{privilegeName}")
  public ApiPrivilege getPrivilege(@PathParam("privilegeName") final String privilegeName) {
    try {
      return toApiPrivilege(getDefaultAuthorizationManager().getPrivilegeByName(privilegeName));
    }
    catch (NoSuchPrivilegeException e) {
      log.debug("Attempt to retrieve privilege '{}' failed, as it wasn't found in the system.", privilegeName, e);
      throw new WebApplicationMessageException(Status.NOT_FOUND, String.format(PRIV_NOT_FOUND, privilegeName),
          MediaType.APPLICATION_JSON);
    }
  }

  @Override
  @DELETE
  @RequiresAuthentication
  @RequiresPermissions("astrum:privileges:delete")
  @Path("{privilegeName}")
  public void deletePrivilege(@PathParam("privilegeName") final String privilegeName) {
    try {
      getDefaultAuthorizationManager().deletePrivilegeByName(privilegeName);
    }
    catch (NoSuchPrivilegeException e) {
      log.debug("Attempt to delete privilege '{}' failed, as it wasn't found in the system.", privilegeName, e);
      throw new WebApplicationMessageException(Status.NOT_FOUND, String.format(PRIV_NOT_FOUND, privilegeName),
          MediaType.APPLICATION_JSON);
    }
    catch (ReadonlyPrivilegeException e) {
      log.debug("Attempt to delete privilege '{}' failed, as it is readonly.", privilegeName, e);
      throw new WebApplicationMessageException(Status.BAD_REQUEST, String.format(PRIV_INTERNAL, privilegeName),
          MediaType.APPLICATION_JSON);
    }
  }

  @Override
  @POST
  @RequiresAuthentication
  @RequiresPermissions("astrum:privileges:create")
  @Path("application")
  public Response createPrivilege(final ApiPrivilegeApplicationRequest privilege) {
    return doCreate(ApplicationPrivilegeDescriptor.TYPE, privilege);
  }

  @Override
  @PUT
  @RequiresAuthentication
  @RequiresPermissions("astrum:privileges:update")
  @Path("application/{privilegeName}")
  public void updatePrivilege(
      @PathParam("privilegeName") final String privilegeName,
      final ApiPrivilegeApplicationRequest privilege)
  {
    doUpdate(privilegeName, ApplicationPrivilegeDescriptor.TYPE, privilege);
  }

  @Override
  @POST
  @RequiresAuthentication
  @RequiresPermissions("astrum:privileges:create")
  @Path("wildcard")
  public Response createPrivilege(final ApiPrivilegeWildcardRequest privilege) {
    return doCreate(WildcardPrivilegeDescriptor.TYPE, privilege);
  }

  @Override
  @PUT
  @RequiresAuthentication
  @RequiresPermissions("astrum:privileges:update")
  @Path("wildcard/{privilegeName}")
  public void updatePrivilege(
      @PathParam("privilegeName") final String privilegeName,
      final ApiPrivilegeWildcardRequest privilege)
  {
    doUpdate(privilegeName, WildcardPrivilegeDescriptor.TYPE, privilege);
  }
}
