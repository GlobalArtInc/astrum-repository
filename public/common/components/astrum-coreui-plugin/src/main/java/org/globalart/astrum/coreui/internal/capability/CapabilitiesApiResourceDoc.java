package org.globalart.astrum.coreui.internal.capability;

import java.util.List;

import jakarta.ws.rs.core.Response;

import io.swagger.v3.oas.annotations.Operation;
import io.swagger.v3.oas.annotations.Parameter;
import io.swagger.v3.oas.annotations.media.ArraySchema;
import io.swagger.v3.oas.annotations.media.Content;
import io.swagger.v3.oas.annotations.media.Schema;
import io.swagger.v3.oas.annotations.responses.ApiResponse;
import io.swagger.v3.oas.annotations.responses.ApiResponses;
import io.swagger.v3.oas.annotations.tags.Tag;

import static org.globalart.astrum.rest.ApiDocConstants.AUTHENTICATION_REQUIRED;
import static org.globalart.astrum.rest.ApiDocConstants.INSUFFICIENT_PERMISSIONS;

/**
 * Swagger documentation for {@link CapabilitiesApiResource}.
 */
@Tag(name = "Capabilities")
public interface CapabilitiesApiResourceDoc
{
  String CAPABILITY_NOT_FOUND = "Capability not found";

  String VALIDATION_FAILED = "Validation failed; the body lists the errors as id and message";

  @Operation(summary = "List capabilities",
      description = "Secret properties are returned as a placeholder, never as their stored value.")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "200", description = "Capabilities returned",
          content = @Content(array = @ArraySchema(schema = @Schema(implementation = CapabilityXO.class)))),
      @ApiResponse(responseCode = "401", description = AUTHENTICATION_REQUIRED),
      @ApiResponse(responseCode = "403", description = INSUFFICIENT_PERMISSIONS)
  })
  List<CapabilityXO> getCapabilities();

  @Operation(summary = "List the capability types that can be created, with the fields of their forms")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "200", description = "Capability types returned",
          content = @Content(array = @ArraySchema(schema = @Schema(implementation = CapabilityTypeXO.class)))),
      @ApiResponse(responseCode = "401", description = AUTHENTICATION_REQUIRED),
      @ApiResponse(responseCode = "403", description = INSUFFICIENT_PERMISSIONS)
  })
  List<CapabilityTypeXO> getCapabilityTypes();

  @Operation(summary = "Get a capability, including its status")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "200", description = "Capability returned",
          content = @Content(schema = @Schema(implementation = CapabilityXO.class))),
      @ApiResponse(responseCode = "401", description = AUTHENTICATION_REQUIRED),
      @ApiResponse(responseCode = "403", description = INSUFFICIENT_PERMISSIONS),
      @ApiResponse(responseCode = "404", description = CAPABILITY_NOT_FOUND)
  })
  CapabilityXO getCapability(@Parameter(description = "Id of the capability", required = true) String id);

  @Operation(summary = "Create a capability",
      description = "Requires type and properties; enabled defaults to true.")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "201", description = "Capability created",
          content = @Content(schema = @Schema(implementation = CapabilityXO.class))),
      @ApiResponse(responseCode = "400", description = VALIDATION_FAILED),
      @ApiResponse(responseCode = "401", description = AUTHENTICATION_REQUIRED),
      @ApiResponse(responseCode = "403", description = INSUFFICIENT_PERMISSIONS)
  })
  Response createCapability(@Parameter(required = true) CapabilityXO capability);

  @Operation(summary = "Update a capability, or enable or disable it",
      description = "Fields left out keep their current value. A secret property sent back as the placeholder "
          + "keeps its stored value.")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "200", description = "Capability updated",
          content = @Content(schema = @Schema(implementation = CapabilityXO.class))),
      @ApiResponse(responseCode = "400", description = VALIDATION_FAILED),
      @ApiResponse(responseCode = "401", description = AUTHENTICATION_REQUIRED),
      @ApiResponse(responseCode = "403", description = INSUFFICIENT_PERMISSIONS),
      @ApiResponse(responseCode = "404", description = CAPABILITY_NOT_FOUND)
  })
  CapabilityXO updateCapability(
      @Parameter(description = "Id of the capability", required = true) String id,
      @Parameter(required = true) CapabilityXO capability);

  @Operation(summary = "Delete a capability")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "204", description = "Capability deleted"),
      @ApiResponse(responseCode = "400", description = "System capabilities cannot be deleted"),
      @ApiResponse(responseCode = "401", description = AUTHENTICATION_REQUIRED),
      @ApiResponse(responseCode = "403", description = INSUFFICIENT_PERMISSIONS),
      @ApiResponse(responseCode = "404", description = CAPABILITY_NOT_FOUND)
  })
  void deleteCapability(@Parameter(description = "Id of the capability", required = true) String id);
}
