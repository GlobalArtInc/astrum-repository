
package org.globalart.astrum.api.rest.selfhosted.security.eula;

import org.globalart.astrum.api.rest.selfhosted.security.eula.model.EulaStatus;

import io.swagger.v3.oas.annotations.tags.Tag;
import io.swagger.v3.oas.annotations.Operation;
import io.swagger.v3.oas.annotations.media.Content;
import io.swagger.v3.oas.annotations.media.Schema;
import io.swagger.v3.oas.annotations.responses.ApiResponse;
import io.swagger.v3.oas.annotations.responses.ApiResponses;

/**
 * Swagger documentation for {@link CommunityEulaApiResource}.
 */
@Tag(name = "Community Edition Eula")
public interface CommunityEulaApiResourceDoc
{
  @Operation(summary = "Get the current Community Eula status.")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "200", description = "Successful response",
          content = @Content(schema = @Schema(implementation = EulaStatus.class)))
  // ASTRUM-46395 TODO: examples= attribute dropped during sweep; restore via
  // @Content(examples = @ExampleObject(value = "{ ... }"))
  })
  EulaStatus getCommunityEulaStatus();

  @Operation(summary = "Set the Community Eula status.")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "204", description = "EULA status set successfully"),
      @ApiResponse(responseCode = "500", description = "Incorrect EULA Status")
  })
  void setEulaAcceptedCE(
      // ASTRUM-46395 TODO: examples on @Parameter moves to a nested @Schema or to @Content;
      // for the spike we accept the loss of the inline example. Restore via
      // @Parameter(content = @Content(examples = @ExampleObject(value = "{...}"))) on the
      // body parameter, or via @RequestBody at the operation level.
      EulaStatus eulaStatus);
}
