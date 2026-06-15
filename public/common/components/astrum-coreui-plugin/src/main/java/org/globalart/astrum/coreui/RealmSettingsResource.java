
package org.globalart.astrum.coreui;

import java.util.List;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Consumes;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.Produces;

import org.globalart.astrum.rest.Resource;
import org.globalart.astrum.security.realm.RealmManager;
import org.globalart.astrum.security.realm.SecurityRealm;
import org.globalart.astrum.security.user.UserManager;

import org.apache.shiro.authz.annotation.RequiresPermissions;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.util.stream.Collectors.toList;
import static jakarta.ws.rs.core.MediaType.APPLICATION_JSON;
import static org.globalart.astrum.security.anonymous.AnonymousHelper.getAuthenticationRealms;
import org.springframework.stereotype.Component;

/**
 * THIS CLASS IS NOT WHAT IT SEEMS! This REST resource is ONLY utilized by the anonymous settings page in UI for
 * displaying the list of realms to select from for pulling the anonymous user from
 * <p>
 * Additionally, this assignment is supposed to be with UserManager impls, but was somehow merged into Realms...
 * <p>
 * The proper security realm endpoint that the UI security pages use is RealmApiResource (it uses the external API)
 *
 * @since 3.19
 */
@Component
@Consumes(APPLICATION_JSON)
@Produces(APPLICATION_JSON)
@Path(RealmSettingsResource.RESOURCE_PATH)
public class RealmSettingsResource
    implements Resource
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  static final String RESOURCE_PATH = "internal/ui/realms";

  private final RealmManager realmManager;

  private final List<String> authenticationRealms;

  @Autowired
  public RealmSettingsResource(
      final RealmManager realmManager,
      List<UserManager> userManagers)
  {
    this.realmManager = checkNotNull(realmManager);
    checkNotNull(userManagers);
    authenticationRealms = getAuthenticationRealms(userManagers);
  }

  @GET
  @Path("/types")
  @RequiresPermissions("astrum:settings:read")
  public List<SecurityRealm> readRealmTypes() {
    return realmManager.getAvailableRealms(true)
        .stream()
        .filter(securityRealm -> authenticationRealms.contains(securityRealm.getId()))
        .collect(toList());
  }
}
