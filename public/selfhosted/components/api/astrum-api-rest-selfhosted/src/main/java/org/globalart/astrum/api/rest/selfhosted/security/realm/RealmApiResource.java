
package org.globalart.astrum.api.rest.selfhosted.security.realm;

import java.util.HashSet;
import java.util.List;
import java.util.Set;
import java.util.stream.Collectors;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Consumes;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.PUT;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.Produces;
import jakarta.ws.rs.core.MediaType;
import jakarta.ws.rs.core.Response.Status;

import org.globalart.astrum.api.rest.selfhosted.security.realm.model.RealmApiXO;
import org.globalart.astrum.rest.Resource;
import org.globalart.astrum.rest.WebApplicationMessageException;
import org.globalart.astrum.security.realm.RealmManager;
import org.globalart.astrum.security.realm.SecurityRealm;

import com.google.common.base.Predicates;
import org.apache.shiro.authz.annotation.RequiresAuthentication;
import org.apache.shiro.authz.annotation.RequiresPermissions;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * REST API for managing Astrum security realms
 *
 * @since 3.20
 */
@Produces(MediaType.APPLICATION_JSON)
@Consumes(MediaType.APPLICATION_JSON)
public class RealmApiResource
    implements RealmApiResourceDoc, Resource
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final RealmManager realmManager;

  @Autowired
  public RealmApiResource(final RealmManager realmManager) {
    this.realmManager = checkNotNull(realmManager);
  }

  @Path("available")
  @GET
  @RequiresAuthentication
  @RequiresPermissions("astrum:settings:read")
  @Override
  public List<RealmApiXO> getRealms() {
    return realmManager.getAvailableRealms().stream().map(RealmApiXO::from).collect(Collectors.toList());
  }

  @Path("active")
  @GET
  @RequiresAuthentication
  @RequiresPermissions("astrum:settings:read")
  @Override
  public List<String> getActiveRealms() {
    return realmManager.getConfiguredRealmIds();
  }

  @Path("active")
  @PUT
  @RequiresAuthentication
  @RequiresPermissions("astrum:settings:update")
  @Override
  public void setActiveRealms(final List<String> realmIds) {
    Set<String> knownRealmIds = new HashSet<>();
    realmManager.getAvailableRealms().stream().map(SecurityRealm::getId).forEach(knownRealmIds::add);

    List<String> unknownRealms =
        realmIds.stream().filter(Predicates.not(knownRealmIds::contains)).collect(Collectors.toList());
    if (!unknownRealms.isEmpty()) {
      log.debug("Request to set realms with unknown IDs: " + unknownRealms);
      throw new WebApplicationMessageException(Status.BAD_REQUEST, "\"Unknown realmIds: " + unknownRealms + "\"");
    }

    realmManager.setConfiguredRealmIds(realmIds);
  }
}
