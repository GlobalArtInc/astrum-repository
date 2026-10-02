
package org.globalart.astrum.api.rest.selfhosted.security.jwt;

import jakarta.ws.rs.core.Response;

import io.swagger.v3.oas.annotations.tags.Tag;
import io.swagger.v3.oas.annotations.Operation;
import io.swagger.v3.oas.annotations.responses.ApiResponse;
import io.swagger.v3.oas.annotations.responses.ApiResponses;

import static org.globalart.astrum.rest.ApiDocConstants.AUTHENTICATION_REQUIRED;
import static org.globalart.astrum.rest.ApiDocConstants.INSUFFICIENT_PERMISSIONS;

/**
 * REST API to reset the stored JWT secret.
 *
 * @since 3.38
 */
@Tag(name = "Security management: JWT")
public interface JwtSecretApiResourceDoc
{
  @Operation(summary = "Reset JWT secret (note that session will be expired for the all logged-in users)")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "401", description = AUTHENTICATION_REQUIRED),
      @ApiResponse(responseCode = "403", description = INSUFFICIENT_PERMISSIONS)
  })
  Response resetSecret();
}
