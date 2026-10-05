
package org.globalart.astrum.api.rest.selfhosted.security.ssrf;

import jakarta.validation.Valid;
import jakarta.validation.constraints.NotNull;

import org.globalart.astrum.api.rest.selfhosted.security.ssrf.model.SsrfProtectionConfigurationXO;

import io.swagger.v3.oas.annotations.tags.Tag;
import io.swagger.v3.oas.annotations.Operation;
import io.swagger.v3.oas.annotations.media.Content;
import io.swagger.v3.oas.annotations.media.Schema;
import io.swagger.v3.oas.annotations.responses.ApiResponse;
import io.swagger.v3.oas.annotations.responses.ApiResponses;

/**
 * Swagger documentation for the SSRF Protection API.
 */
@Tag(name = "Security Management: SSRF Protection")
public interface SsrfProtectionApiResourceDoc
{
  String EXAMPLE_BODY =
      "{\"enabled\": true, \"allowedIPs\": [\"10.0.0.50\", \"192.168.1.100\"], \"allowedDomains\": [\"internal.corp.com\", \"registry.local\"]}";

  @Operation(summary = "Get SSRF protection settings")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "200", description = "SSRF protection settings returned",
          content = @Content(schema = @Schema(implementation = SsrfProtectionConfigurationXO.class))),
      @ApiResponse(responseCode = "401", description = "Authentication required"),
      @ApiResponse(responseCode = "403", description = "Insufficient permissions")
  })
  SsrfProtectionConfigurationXO getConfiguration();

  @Operation(summary = "Update SSRF protection settings")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "200", description = "SSRF protection settings updated",
          content = @Content(schema = @Schema(implementation = SsrfProtectionConfigurationXO.class))),
      @ApiResponse(responseCode = "400", description = "Invalid configuration"),
      @ApiResponse(responseCode = "401", description = "Authentication required"),
      @ApiResponse(responseCode = "403", description = "Insufficient permissions")
  })
  SsrfProtectionConfigurationXO updateConfiguration(@NotNull @Valid SsrfProtectionConfigurationXO configuration);
}
