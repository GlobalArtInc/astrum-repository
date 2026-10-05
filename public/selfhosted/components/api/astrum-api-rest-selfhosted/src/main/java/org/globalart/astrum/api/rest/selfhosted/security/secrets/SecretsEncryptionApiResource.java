
package org.globalart.astrum.api.rest.selfhosted.security.secrets;

import java.util.Map;
import jakarta.validation.Valid;
import jakarta.ws.rs.Consumes;
import jakarta.ws.rs.PUT;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.Produces;
import jakarta.ws.rs.core.Response;
import jakarta.ws.rs.core.Response.Status;

import org.globalart.astrum.api.rest.selfhosted.security.secrets.model.ReEncryptionRequestApiXO;
import org.globalart.astrum.crypto.secrets.MissingKeyException;
import org.globalart.astrum.crypto.secrets.ReEncryptService;
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
public abstract class SecretsEncryptionApiResource
    implements Resource, SecretsEncryptionApiResourceDoc
{
  public static final String RESOURCE_PATH = "/secrets/encryption";

  private final ReEncryptService reEncryptService;

  protected SecretsEncryptionApiResource(final ReEncryptService reEncryptService) {
    this.reEncryptService = checkNotNull(reEncryptService);
  }

  @PUT
  @Override
  @Path("/re-encrypt")
  @RequiresAuthentication
  @RequiresPermissions("astrum:*")
  public Response reEncrypt(@Valid final ReEncryptionRequestApiXO request) {
    try {
      String taskId = reEncryptService.submitReEncryption(request.getSecretKeyId(), request.getNotifyEmail());
      Map<String, Object> response = ImmutableMap.of("status", Status.ACCEPTED.getStatusCode(), "message",
          "Task submitted. ID: " + taskId);
      return Response
          .status(Status.ACCEPTED)
          .entity(response)
          .type(APPLICATION_JSON)
          .build();
    }
    catch (MissingKeyException | ReEncryptionNotSupportedException ex) {
      throw new WebApplicationMessageException(Status.BAD_REQUEST, ex.getMessage(), APPLICATION_JSON);
    }
    catch (IllegalStateException ex) {
      throw new WebApplicationMessageException(Status.CONFLICT, ex.getMessage(), APPLICATION_JSON);
    }
  }

}
