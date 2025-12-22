
package org.globalart.astrum.security.role.rest;

import java.util.List;

import jakarta.validation.Valid;
import jakarta.validation.constraints.NotNull;

import org.globalart.astrum.security.authz.NoSuchAuthorizationManagerException;

import io.swagger.v3.oas.annotations.tags.Tag;
import io.swagger.v3.oas.annotations.Operation;
import io.swagger.v3.oas.annotations.Parameter;
import io.swagger.v3.oas.annotations.media.ArraySchema;
import io.swagger.v3.oas.annotations.media.Content;
import io.swagger.v3.oas.annotations.media.Schema;
import io.swagger.v3.oas.annotations.responses.ApiResponse;
import io.swagger.v3.oas.annotations.responses.ApiResponses;
import jakarta.validation.constraints.NotEmpty;

/**
 * Swagger documentation for {@link RoleApiResource}
 *
 * @since 3.19
 */
@Tag(name = "Security management: roles")
public interface RoleApiResourceDoc
{
  @Operation(summary = "List roles")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "200", description = "Roles returned",
          content = @Content(array = @ArraySchema(schema = @Schema(implementation = RoleXOResponse.class)))),
      @ApiResponse(responseCode = "400", description = "The specified source does not exist"),
      @ApiResponse(responseCode = "403", description = "Insufficient permissions to read roles")})
  List<RoleXOResponse> getRoles(
      @Parameter(
          description = "The id of the user source to filter the roles by, if supplied. Otherwise roles from all user sources will be returned.") final String source);

  @Operation(summary = "Create role")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "200", description = "Role created",
          content = @Content(schema = @Schema(implementation = RoleXOResponse.class))),
      @ApiResponse(responseCode = "400",
          description = "The specified source does not exist, the role already exists, a referenced role or privilege was not found, or the role contains itself"),
      @ApiResponse(responseCode = "403", description = "Insufficient permissions to create role")})
  RoleXOResponse create(
      @Parameter(
          description = "The id of the user source to create the role in. Available sources can be fetched using the 'User Sources' endpoint.") final String source,
      @Parameter(description = "A role configuration",
          required = true) @NotNull @Valid final RoleXORequest roleXO) throws NoSuchAuthorizationManagerException;

  @Operation(summary = "Get role")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "200", description = "Role returned",
          content = @Content(schema = @Schema(implementation = RoleXOResponse.class))),
      @ApiResponse(responseCode = "400", description = "The specified source does not exist"),
      @ApiResponse(responseCode = "403", description = "Insufficient permissions to read roles"),
      @ApiResponse(responseCode = "404", description = "Role not found")})
  RoleXOResponse getRole(
      @Parameter(
          description = "The id of the user source to filter the roles by. Available sources can be fetched using the 'User Sources' endpoint." /*
                                                                                                                                                 * ASTRUM
                                                                                                                                                 * -
                                                                                                                                                 * 46395
                                                                                                                                                 * TODO:
                                                                                                                                                 * defaultValue
                                                                                                                                                 * moved
                                                                                                                                                 * to @Schema
                                                                                                                                                 * in
                                                                                                                                                 * OpenAPI
                                                                                                                                                 * 3
                                                                                                                                                 * .
                                                                                                                                                 * x: @Parameter
                                                                                                                                                 * (
                                                                                                                                                 * schema
                                                                                                                                                 * = @Schema
                                                                                                                                                 * (
                                                                                                                                                 * defaultValue
                                                                                                                                                 * =
                                                                                                                                                 * DEFAULT_SOURCE
                                                                                                                                                 * )
                                                                                                                                                 * )
                                                                                                                                                 */) final String source,
      @Parameter(description = "The id of the role to get", required = true) @NotEmpty final String id);

  @Operation(summary = "Update role")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "400",
          description = "The specified source does not exist, a referenced role or privilege was not found, the role is read-only, or the role contains itself"),
      @ApiResponse(responseCode = "403", description = "Insufficient permissions to update role"),
      @ApiResponse(responseCode = "404", description = "Role not found")})
  void update(
      @Parameter(description = "The id of the role to update", required = true) @NotEmpty final String id,
      @Parameter(
          description = "The id of the user source that contains the role. Available sources can be fetched using the 'User Sources' endpoint.") final String source,
      @Parameter(description = "A role configuration",
          required = true) @NotNull @Valid final RoleXORequest roleXO) throws NoSuchAuthorizationManagerException;

  @Operation(summary = "Delete role")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "400", description = "The specified source does not exist, or the role is read-only"),
      @ApiResponse(responseCode = "403", description = "Insufficient permissions to delete role"),
      @ApiResponse(responseCode = "404", description = "Role not found"),
      @ApiResponse(responseCode = "204", description = "Success")})
  void delete(
      @Parameter(description = "The id of the role to delete", required = true) @NotEmpty final String id,
      @Parameter(
          description = "The id of the user source that contains the role. Available sources can be fetched using the 'User Sources' endpoint.") final String source) throws NoSuchAuthorizationManagerException;

  @Operation(summary = "Get assignable roles")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "200", description = "Assignable roles returned",
          content = @Content(array = @ArraySchema(schema = @Schema(implementation = RoleXOResponse.class)))),
      @ApiResponse(responseCode = "403", description = "Insufficient permissions to read roles"),
      @ApiResponse(responseCode = "400", description = "The specified source does not exist")})
  List<RoleXOResponse> getAssignableRoles(
      @Parameter(
          description = "The id of the user source to filter the roles by, if supplied. Otherwise roles from default user source will be returned. Available sources can be fetched using the 'User Sources' endpoint.") final String source);
}
