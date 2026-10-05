
package org.globalart.astrum.api.rest.selfhosted.security.secrets;

import jakarta.validation.Valid;
import jakarta.ws.rs.core.Response;

import org.globalart.astrum.api.rest.selfhosted.security.secrets.model.ReEncryptionRequestApiXO;

import io.swagger.v3.oas.annotations.tags.Tag;
import io.swagger.v3.oas.annotations.Operation;
import io.swagger.v3.oas.annotations.responses.ApiResponse;
import io.swagger.v3.oas.annotations.responses.ApiResponses;

/**
 * REST API to set a new encryption key and re-encrypt secrets
 */
@Tag(name = "Security management: secrets encryption")
public interface SecretsEncryptionApiResourceDoc
{
  @Operation(summary = "Re-encrypt secrets using the specified key",
      description = "Ensure all nodes have access to the key, and they use the same key")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "202", description = "Re-encrypt task successfully submitted"),
      @ApiResponse(responseCode = "400", description = "Invalid request. See the response for more information. " +
          "Possible causes: The key is not available to all nodes, upgrade needed or empty key id."),
      @ApiResponse(responseCode = "403", description = "Insufficient permissions to re-encrypt secrets"),
      @ApiResponse(responseCode = "409", description = "Re-encryption task in progress.")
  })
  Response reEncrypt(@Valid final ReEncryptionRequestApiXO reEncryptionRequestApiXO);
}
