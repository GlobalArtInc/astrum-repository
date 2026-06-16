
package org.globalart.astrum.coreui.internal.privileges;

import java.util.List;
import java.util.Map;
import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Consumes;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.Produces;

import org.globalart.astrum.common.QualifierUtil;
import org.globalart.astrum.rest.Resource;
import org.globalart.astrum.security.privilege.PrivilegeDescriptor;

import org.apache.shiro.authz.annotation.RequiresAuthentication;
import org.apache.shiro.authz.annotation.RequiresPermissions;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.util.stream.Collectors.toList;
import static jakarta.ws.rs.core.MediaType.APPLICATION_JSON;
import org.springframework.stereotype.Component;

@Component
@Consumes(APPLICATION_JSON)
@Produces(APPLICATION_JSON)
@Path("/internal/ui/privileges")
public class PrivilegesUIResource
    implements Resource
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final Map<String, PrivilegeDescriptor> privilegeDescriptors;

  @Autowired
  public PrivilegesUIResource(final List<PrivilegeDescriptor> privilegeDescriptorsList) {
    this.privilegeDescriptors = QualifierUtil.buildQualifierBeanMap(checkNotNull(privilegeDescriptorsList));
  }

  @RequiresAuthentication
  @RequiresPermissions("astrum:privileges:read")
  @GET
  @Path("/types")
  public List<PrivilegesTypesUIResponse> listPrivilegesTypes() {
    return privilegeDescriptors.entrySet()
        .stream()
        .map(PrivilegesTypesUIResponse::new)
        .collect(toList());
  }
}
