
package org.globalart.astrum.api.rest.selfhosted.security.anonymous;

import jakarta.validation.Valid;

import org.globalart.astrum.api.rest.selfhosted.security.anonymous.model.AnonymousAccessSettingsXO;

import io.swagger.v3.oas.annotations.tags.Tag;
import io.swagger.v3.oas.annotations.Operation;
import io.swagger.v3.oas.annotations.media.Content;
import io.swagger.v3.oas.annotations.media.Schema;
import io.swagger.v3.oas.annotations.responses.ApiResponse;
import io.swagger.v3.oas.annotations.responses.ApiResponses;

/**
 * @since 3.24
 */
@Tag(name = "Security Management: Anonymous Access")
public interface AnonymousAccessApiResourceDoc
{
  @Operation(summary = "Get Anonymous Access settings")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "200", description = "Anonymous access settings returned",
          content = @Content(schema = @Schema(implementation = AnonymousAccessSettingsXO.class))),
      @ApiResponse(responseCode = "403", description = "Insufficient permissions to update settings")
  })
  AnonymousAccessSettingsXO read();

  @Operation(summary = "Update Anonymous Access settings")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "200", description = "Anonymous access settings updated",
          content = @Content(schema = @Schema(implementation = AnonymousAccessSettingsXO.class))),
      @ApiResponse(responseCode = "403", description = "Insufficient permissions to update settings")
  })
  AnonymousAccessSettingsXO update(@Valid AnonymousAccessSettingsXO anonymousXO);
}
