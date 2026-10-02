
package org.globalart.astrum.api.rest.selfhosted.security.eula;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Path;

import org.globalart.astrum.kv.GlobalKeyValueStore;

import static org.globalart.astrum.rest.APIConstants.V1_API_PREFIX;
import org.springframework.stereotype.Component;

/**
 * v1 api for community eula
 */
@Component
@Path(CommunityEulaApiResourceV1.RESOURCE_URI)
public class CommunityEulaApiResourceV1
    extends CommunityEulaApiResource
{
  public static final String RESOURCE_URI = V1_API_PREFIX + "/system/eula";

  @Autowired
  public CommunityEulaApiResourceV1(final GlobalKeyValueStore globalKeyValueStore) {
    super(globalKeyValueStore);
  }
}
