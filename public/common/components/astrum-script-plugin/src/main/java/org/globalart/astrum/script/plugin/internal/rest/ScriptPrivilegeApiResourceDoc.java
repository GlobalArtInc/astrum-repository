
package org.globalart.astrum.script.plugin.internal.rest;

import jakarta.validation.Valid;
import jakarta.validation.constraints.NotNull;
import jakarta.ws.rs.core.Response;

import org.globalart.astrum.security.internal.rest.AstrumSecurityApiConstants;

import io.swagger.v3.oas.annotations.tags.Tag;
import io.swagger.v3.oas.annotations.Operation;
import io.swagger.v3.oas.annotations.Parameter;
import io.swagger.v3.oas.annotations.responses.ApiResponse;
import io.swagger.v3.oas.annotations.responses.ApiResponses;

/**
 * @since 3.19
 */
@Tag(name = "Security management: privileges")
public interface ScriptPrivilegeApiResourceDoc
{
  @Operation(summary = "Create a script type privilege.")
  @ApiResponses(
      value = {@ApiResponse(responseCode = "400", description = AstrumSecurityApiConstants.PRIVILEGE_MISCONFIGURED),
          @ApiResponse(responseCode = "403", description = AstrumSecurityApiConstants.INVALID_PERMISSIONS),
          @ApiResponse(responseCode = "201", description = AstrumSecurityApiConstants.PRIVILEGE_CREATED)})
  Response createPrivilege(
      @Parameter(description = "The privilege to create.") @NotNull @Valid final ApiPrivilegeScriptRequest privilege);

  @Operation(summary = "Update a script type privilege.")
  @ApiResponses(
      value = {@ApiResponse(responseCode = "400", description = AstrumSecurityApiConstants.PRIVILEGE_MISCONFIGURED),
          @ApiResponse(responseCode = "403", description = AstrumSecurityApiConstants.INVALID_PERMISSIONS),
          @ApiResponse(responseCode = "404", description = AstrumSecurityApiConstants.PRIVILEGE_NOT_FOUND),
          @ApiResponse(responseCode = "409", description = AstrumSecurityApiConstants.PRIVILEGE_CONFLICT),
          @ApiResponse(responseCode = "204", description = AstrumSecurityApiConstants.SUCCESS)})
  void updatePrivilege(
      @Parameter(description = "The name of the privilege to update.") @NotNull final String privilegeName,
      @Parameter(description = "The privilege to update.") @NotNull @Valid final ApiPrivilegeScriptRequest privilege);
}
