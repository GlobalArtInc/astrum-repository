
package org.globalart.astrum.api.rest.selfhosted.security.eula;

import java.util.Map;
import java.util.Optional;
import jakarta.ws.rs.Consumes;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.POST;
import jakarta.ws.rs.Produces;
import jakarta.ws.rs.core.MediaType;

import org.globalart.astrum.api.rest.selfhosted.security.eula.model.EulaStatus;
import org.globalart.astrum.kv.GlobalKeyValueStore;
import org.globalart.astrum.kv.AstrumKeyValue;
import org.globalart.astrum.kv.ValueType;
import org.globalart.astrum.rest.Resource;

import org.apache.shiro.authz.annotation.RequiresAuthentication;
import org.apache.shiro.authz.annotation.RequiresPermissions;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class CommunityEulaApiResource
    implements Resource, CommunityEulaApiResourceDoc
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final String EULA_KEY = "astrum.community.eula.accepted";

  private final GlobalKeyValueStore globalKeyValueStore;

  public CommunityEulaApiResource(final GlobalKeyValueStore globalKeyValueStore) {
    this.globalKeyValueStore = globalKeyValueStore;
  }

  @Override
  @RequiresPermissions("astrum:*")
  @RequiresAuthentication
  @GET
  @Produces(MediaType.APPLICATION_JSON)
  public EulaStatus getCommunityEulaStatus() {
    Optional<AstrumKeyValue> eulaStatusOptional = globalKeyValueStore.getKey(EULA_KEY);
    EulaStatus eulaStatus = new EulaStatus();
    if (eulaStatusOptional.isPresent()) {
      AstrumKeyValue eulaStatusKeyValue = eulaStatusOptional.get();
      Boolean accepted = (Boolean) eulaStatusKeyValue.value().get("accepted");
      if (accepted != null) {
        eulaStatus.setAccepted(accepted);
      }
    }
    else {
      eulaStatus.setAccepted(false);
    }
    eulaStatus.setDisclaimer(EulaStatus.EXPECTED_DISCLAIMER);
    return eulaStatus;
  }

  @Override
  @POST
  @RequiresAuthentication
  @RequiresPermissions("astrum:*")
  @Consumes(MediaType.APPLICATION_JSON)
  public void setEulaAcceptedCE(EulaStatus eulaStatus) {
    boolean validDisclaimer = eulaStatus.hasExpectedDisclaimer();
    boolean isAlreadyAccepted = getCommunityEulaStatus().isAccepted();
    boolean isSettingFalse = !eulaStatus.isAccepted();

    if (!validDisclaimer) {
      throw new IllegalArgumentException("Invalid EULA disclaimer");
    }
    else if (isSettingFalse) {
      throw new IllegalArgumentException("EULA must be accepted");
    }
    else if (isAlreadyAccepted) {
      log.debug("EULA was already accepted, ignoring request");
      return;
    }

    if (eulaStatus.hasExpectedDisclaimer() && eulaStatus.isAccepted()) {
      AstrumKeyValue kv = new AstrumKeyValue();
      kv.setKey(EULA_KEY);
      kv.setType(ValueType.OBJECT);
      kv.setValue(Map.of("accepted", eulaStatus.isAccepted()));
      globalKeyValueStore.setKey(kv);
    }
  }
}
