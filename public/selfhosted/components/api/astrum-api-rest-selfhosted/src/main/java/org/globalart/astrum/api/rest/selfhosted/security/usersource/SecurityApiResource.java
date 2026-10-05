

package org.globalart.astrum.api.rest.selfhosted.security.usersource;

import java.util.List;
import java.util.Map;
import java.util.stream.Collectors;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.Produces;
import jakarta.ws.rs.core.MediaType;

import org.globalart.astrum.api.rest.selfhosted.security.usersource.model.ApiUserSource;
import org.globalart.astrum.common.QualifierUtil;
import org.globalart.astrum.rest.Resource;
import org.globalart.astrum.security.user.ConfiguredUsersUserManager;
import org.globalart.astrum.security.user.UserManager;

import org.apache.shiro.authz.annotation.RequiresAuthentication;
import org.apache.shiro.authz.annotation.RequiresPermissions;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * @since 3.17
 */
@Produces(MediaType.APPLICATION_JSON)
public class SecurityApiResource
    implements Resource, SecurityApiResourceDoc
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final Map<String, UserManager> userManagers;

  @Autowired
  public SecurityApiResource(final List<UserManager> userManagersList) {
    this.userManagers = QualifierUtil.buildQualifierBeanMap(userManagersList);
  }

  @Override
  @GET
  @Path("user-sources")
  @RequiresAuthentication
  @RequiresPermissions("astrum:users:read")
  public List<ApiUserSource> getUserSources() {
    return userManagers.values()
        .stream()
        .filter(um -> !ConfiguredUsersUserManager.SOURCE.equals(um.getSource()))
        .map(um -> new ApiUserSource(um))
        .collect(Collectors.toList());
  }
}
