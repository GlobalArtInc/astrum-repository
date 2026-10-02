
package org.globalart.astrum.api.rest.selfhosted.security.realm;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Path;

import org.globalart.astrum.rest.APIConstants;
import org.globalart.astrum.security.realm.RealmManager;
import org.springframework.stereotype.Component;

/**
 * @since 3.26
 */
@Component
@Path(RealmApiResourceV1.RESOURCE_URL)
public class RealmApiResourceV1
    extends RealmApiResource
{
  static final String RESOURCE_URL = APIConstants.V1_API_PREFIX + "/security/realms";

  @Autowired
  public RealmApiResourceV1(final RealmManager realmManager) {
    super(realmManager);
  }
}
