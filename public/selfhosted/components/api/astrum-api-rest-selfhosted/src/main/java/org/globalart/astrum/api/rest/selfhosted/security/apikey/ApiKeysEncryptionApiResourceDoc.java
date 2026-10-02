
package org.globalart.astrum.api.rest.selfhosted.security.apikey;

import jakarta.validation.Valid;
import jakarta.ws.rs.core.Response;

import org.globalart.astrum.api.rest.selfhosted.security.apikey.model.ApiKeysReEncryptionRequestApiXO;

import io.swagger.v3.oas.annotations.tags.Tag;
import io.swagger.v3.oas.annotations.Operation;
import io.swagger.v3.oas.annotations.responses.ApiResponse;
import io.swagger.v3.oas.annotations.responses.ApiResponses;

/**
 * REST API to re-encrypt api keys principals
 */
@Tag(name = "Security management: api keys principals encryption")
public interface ApiKeysEncryptionApiResourceDoc
{
  @Operation(summary = "Re-encrypt api keys principals using the specified configuration",
      description = "Ensure all nodes have access to the key, and they use the same key")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "202", description = "Re-encrypt task successfully submitted"),
      @ApiResponse(responseCode = "400", description = "Invalid request. See the response for more information. " +
          "Possible causes: The key is not available to all nodes, upgrade needed or empty key id."),
      @ApiResponse(responseCode = "403", description = "Insufficient permissions to re-encrypt secrets"),
      @ApiResponse(responseCode = "409", description = "Re-encryption task in progress.")
  })
  Response reEncrypt(@Valid final ApiKeysReEncryptionRequestApiXO reEncryptionRequestApiXO);
}
