
package org.globalart.astrum.api.rest.selfhosted.security.apikey;

import java.util.Map;
import javax.annotation.Nullable;
import jakarta.ws.rs.Consumes;
import jakarta.ws.rs.PUT;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.Produces;
import jakarta.ws.rs.core.Response;
import jakarta.ws.rs.core.Response.Status;

import org.globalart.astrum.api.rest.selfhosted.security.apikey.model.ApiKeysReEncryptionRequestApiXO;
import org.globalart.astrum.crypto.apikey.ApiKeysReEncryptService;
import org.globalart.astrum.crypto.secrets.ReEncryptionNotSupportedException;
import org.globalart.astrum.rest.Resource;
import org.globalart.astrum.rest.WebApplicationMessageException;

import com.google.common.collect.ImmutableMap;
import org.apache.shiro.authz.annotation.RequiresAuthentication;
import org.apache.shiro.authz.annotation.RequiresPermissions;

import static com.google.common.base.Preconditions.checkNotNull;
import static jakarta.ws.rs.core.MediaType.APPLICATION_JSON;

@Consumes(APPLICATION_JSON)
@Produces(APPLICATION_JSON)
public abstract class ApiKeysEncryptionApiResource
    implements Resource, ApiKeysEncryptionApiResourceDoc
{
  public static final String RESOURCE_PATH = "/apikeys/encryption";

  private final ApiKeysReEncryptService apiKeysReEncryptService;

  protected ApiKeysEncryptionApiResource(final ApiKeysReEncryptService apiKeysReEncryptService) {
    this.apiKeysReEncryptService = checkNotNull(apiKeysReEncryptService);
  }

  @PUT
  @Override
  @Path("/re-encrypt")
  @RequiresAuthentication
  @RequiresPermissions("astrum:*")
  public Response reEncrypt(@Nullable final ApiKeysReEncryptionRequestApiXO request) {
    String algorithmForDecryption = request != null ? request.getAlgorithmForDecryption() : null;
    Integer iterationsForDecryption = request != null ? request.getIterationsForDecryption() : null;
    String notifyEmail = request != null ? request.getNotifyEmail() : null;
    try {
      String taskId =
          apiKeysReEncryptService.submitReEncryption(algorithmForDecryption, iterationsForDecryption, notifyEmail);
      Map<String, Object> response = ImmutableMap.of("status", Status.ACCEPTED.getStatusCode(), "message",
          "Task submitted. ID: " + taskId);
      return Response
          .status(Status.ACCEPTED)
          .entity(response)
          .type(APPLICATION_JSON)
          .build();
    }
    catch (ReEncryptionNotSupportedException ex) {
      throw new WebApplicationMessageException(Status.BAD_REQUEST, ex.getMessage(), APPLICATION_JSON);
    }
    catch (IllegalStateException ex) {
      throw new WebApplicationMessageException(Status.CONFLICT, ex.getMessage(), APPLICATION_JSON);
    }
  }
}
