
package org.globalart.astrum.script.plugin.internal.rest;

import java.util.List;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Consumes;
import jakarta.ws.rs.POST;
import jakarta.ws.rs.PUT;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.PathParam;
import jakarta.ws.rs.Produces;
import jakarta.ws.rs.core.Response;

import org.globalart.astrum.rest.Resource;
import org.globalart.astrum.script.plugin.internal.security.ScriptPrivilegeDescriptor;
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
public class ScriptPrivilegeApiResource
    extends PrivilegeApiResourceSupport
    implements Resource, ScriptPrivilegeApiResourceDoc
{
  @Autowired
  public ScriptPrivilegeApiResource(
      final SecuritySystem securitySystem,
      final SecurityHelper securityHelper,
      final List<PrivilegeDescriptor> privilegeDescriptorsList)
  {
    super(securitySystem, securityHelper, privilegeDescriptorsList);
  }

  @Override
  @POST
  @RequiresAuthentication
  @RequiresPermissions("astrum:privileges:create")
  @Path("script")
  public Response createPrivilege(final ApiPrivilegeScriptRequest privilege) {
    return doCreate(ScriptPrivilegeDescriptor.TYPE, privilege);
  }

  @Override
  @PUT
  @RequiresAuthentication
  @RequiresPermissions("astrum:privileges:update")
  @Path("script/{privilegeName}")
  public void updatePrivilege(
      @PathParam("privilegeName") final String privilegeName,
      final ApiPrivilegeScriptRequest privilege)
  {
    doUpdate(privilegeName, ScriptPrivilegeDescriptor.TYPE, privilege);
  }
}
