
package org.globalart.astrum.api.rest.selfhosted.security.apikey;

import jakarta.ws.rs.Path;

import org.globalart.astrum.crypto.apikey.ApiKeysReEncryptService;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.rest.APIConstants.V1_API_PREFIX;

@Component
@Path(ApiKeysEncryptionApiResourceV1.RESOURCE_URI)
public class ApiKeysEncryptionApiResourceV1
    extends ApiKeysEncryptionApiResource
{
  public static final String RESOURCE_URI = V1_API_PREFIX + RESOURCE_PATH;

  @Autowired
  public ApiKeysEncryptionApiResourceV1(final ApiKeysReEncryptService apiKeysReEncryptService) {
    super(apiKeysReEncryptService);
  }
}
