
package org.globalart.astrum.api.rest.selfhosted.security.realm;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Path;

import org.globalart.astrum.rest.APIConstants;
import org.globalart.astrum.security.realm.RealmManager;

import io.swagger.v3.oas.annotations.Hidden;
import org.springframework.stereotype.Component;

/**
 * @since 3.26
 * @deprecated beta prefix is being phased out, prefer starting new APIs with {@link APIConstants#V1_API_PREFIX} instead
 */
@Hidden
@Component
@Path(RealmApiResourceBeta.RESOURCE_URL)
@Deprecated
public class RealmApiResourceBeta
    extends RealmApiResource
{
  static final String RESOURCE_URL = APIConstants.BETA_API_PREFIX + "/security/realms";

  @Autowired
  public RealmApiResourceBeta(final RealmManager realmManager) {
    super(realmManager);
  }
}
