package org.globalart.astrum.coreui.internal.http;

import io.swagger.v3.oas.annotations.Operation;
import io.swagger.v3.oas.annotations.media.Content;
import io.swagger.v3.oas.annotations.media.Schema;
import io.swagger.v3.oas.annotations.responses.ApiResponse;
import io.swagger.v3.oas.annotations.responses.ApiResponses;
import io.swagger.v3.oas.annotations.tags.Tag;

import static org.globalart.astrum.rest.ApiDocConstants.AUTHENTICATION_REQUIRED;
import static org.globalart.astrum.rest.ApiDocConstants.INSUFFICIENT_PERMISSIONS;

/**
 * Swagger documentation for {@link HttpSettingsApiResource}.
 */
@Tag(name = "HTTP settings")
public interface HttpSettingsApiResourceDoc
{
  @Operation(summary = "Get the HTTP settings",
      description = "Proxy passwords are returned as a placeholder, never as their stored value.")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "200", description = "HTTP settings returned",
          content = @Content(schema = @Schema(implementation = HttpSettingsApiXO.class))),
      @ApiResponse(responseCode = "401", description = AUTHENTICATION_REQUIRED),
      @ApiResponse(responseCode = "403", description = INSUFFICIENT_PERMISSIONS)
  })
  HttpSettingsApiXO getHttpSettings();

  @Operation(summary = "Update the HTTP settings",
      description = "Replaces all settings. A proxy password sent as the placeholder keeps the stored one.")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "200", description = "HTTP settings updated",
          content = @Content(schema = @Schema(implementation = HttpSettingsApiXO.class))),
      @ApiResponse(responseCode = "400", description = "Invalid settings; the body lists the errors as id and message"),
      @ApiResponse(responseCode = "401", description = AUTHENTICATION_REQUIRED),
      @ApiResponse(responseCode = "403", description = INSUFFICIENT_PERMISSIONS)
  })
  HttpSettingsApiXO updateHttpSettings(HttpSettingsApiXO settings);
}
