
package org.globalart.astrum.api.rest.selfhosted.security.secrets;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Path;

import org.globalart.astrum.crypto.secrets.ReEncryptService;

import static org.globalart.astrum.rest.APIConstants.V1_API_PREFIX;
import org.springframework.stereotype.Component;

@Component
@Path(SecretsEncryptionApiResourceV1.RESOURCE_URI)
public class SecretsEncryptionApiResourceV1
    extends SecretsEncryptionApiResource
{
  public static final String RESOURCE_URI = V1_API_PREFIX + RESOURCE_PATH;

  @Autowired
  public SecretsEncryptionApiResourceV1(final ReEncryptService reEncryptService) {
    super(reEncryptService);
  }
}
