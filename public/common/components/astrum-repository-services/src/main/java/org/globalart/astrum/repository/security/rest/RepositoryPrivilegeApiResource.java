
package org.globalart.astrum.repository.security.rest;

import java.util.List;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Consumes;
import jakarta.ws.rs.POST;
import jakarta.ws.rs.PUT;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.PathParam;
import jakarta.ws.rs.Produces;
import jakarta.ws.rs.core.Response;

import org.globalart.astrum.repository.security.RepositoryAdminPrivilegeDescriptor;
import org.globalart.astrum.repository.security.RepositoryContentSelectorPrivilegeDescriptor;
import org.globalart.astrum.repository.security.RepositoryViewPrivilegeDescriptor;
import org.globalart.astrum.rest.Resource;
import org.globalart.astrum.security.SecurityHelper;
import org.globalart.astrum.security.SecuritySystem;
import org.globalart.astrum.security.privilege.PrivilegeDescriptor;
import org.globalart.astrum.security.privilege.rest.PrivilegeApiResourceSupport;

import org.apache.shiro.authz.annotation.RequiresAuthentication;
import org.apache.shiro.authz.annotation.RequiresPermissions;

import static jakarta.ws.rs.core.MediaType.APPLICATION_JSON;

/**
 * @since 3.19
 */
@Consumes(APPLICATION_JSON)
@Produces(APPLICATION_JSON)
public class RepositoryPrivilegeApiResource
    extends PrivilegeApiResourceSupport
    implements Resource, RepositoryPrivilegeApiResourceDoc
{
  @Autowired
  public RepositoryPrivilegeApiResource(
      final SecuritySystem securitySystem,
      final SecurityHelper securityHelper,
      final List<PrivilegeDescriptor> privilegeDescriptors)
  {
    super(securitySystem, securityHelper, privilegeDescriptors);
  }

  @Override
  @POST
  @RequiresAuthentication
  @RequiresPermissions("astrum:privileges:create")
  @Path("repository-admin")
  public Response createPrivilege(final ApiPrivilegeRepositoryAdminRequest privilege) {
    return doCreate(RepositoryAdminPrivilegeDescriptor.TYPE, privilege);
  }

  @Override
  @PUT
  @RequiresAuthentication
  @RequiresPermissions("astrum:privileges:update")
  @Path("repository-admin/{privilegeName}")
  public void updatePrivilege(
      @PathParam("privilegeName") final String privilegeName,
      final ApiPrivilegeRepositoryAdminRequest privilege)
  {
    doUpdate(privilegeName, RepositoryAdminPrivilegeDescriptor.TYPE, privilege);
  }

  @Override
  @POST
  @RequiresAuthentication
  @RequiresPermissions("astrum:privileges:create")
  @Path("repository-view")
  public Response createPrivilege(final ApiPrivilegeRepositoryViewRequest privilege) {
    return doCreate(RepositoryViewPrivilegeDescriptor.TYPE, privilege);
  }

  @Override
  @PUT
  @RequiresAuthentication
  @RequiresPermissions("astrum:privileges:update")
  @Path("repository-view/{privilegeName}")
  public void updatePrivilege(
      @PathParam("privilegeName") final String privilegeName,
      final ApiPrivilegeRepositoryViewRequest privilege)
  {
    doUpdate(privilegeName, RepositoryViewPrivilegeDescriptor.TYPE, privilege);
  }

  @Override
  @POST
  @RequiresAuthentication
  @RequiresPermissions("astrum:privileges:create")
  @Path("repository-content-selector")
  public Response createPrivilege(final ApiPrivilegeRepositoryContentSelectorRequest privilege) {
    return doCreate(RepositoryContentSelectorPrivilegeDescriptor.TYPE, privilege);
  }

  @Override
  @PUT
  @RequiresAuthentication
  @RequiresPermissions("astrum:privileges:update")
  @Path("repository-content-selector/{privilegeName}")
  public void updatePrivilege(
      @PathParam("privilegeName") final String privilegeName,
      final ApiPrivilegeRepositoryContentSelectorRequest privilege)
  {
    doUpdate(privilegeName, RepositoryContentSelectorPrivilegeDescriptor.TYPE, privilege);
  }
}
