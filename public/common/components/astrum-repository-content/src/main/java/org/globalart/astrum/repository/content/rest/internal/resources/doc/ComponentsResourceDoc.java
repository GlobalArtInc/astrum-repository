
package org.globalart.astrum.repository.content.rest.internal.resources.doc;

import java.io.IOException;

import jakarta.servlet.http.HttpServletRequest;

import org.globalart.astrum.repository.rest.api.ComponentXO;
import org.globalart.astrum.rest.Page;

import io.swagger.v3.oas.annotations.tags.Tag;
import io.swagger.v3.oas.annotations.Operation;
import io.swagger.v3.oas.annotations.Parameter;
import io.swagger.v3.oas.annotations.media.Content;
import io.swagger.v3.oas.annotations.media.Schema;
import io.swagger.v3.oas.annotations.responses.ApiResponse;
import io.swagger.v3.oas.annotations.responses.ApiResponses;
import org.jboss.resteasy.annotations.providers.multipart.MultipartForm;

/**
 * Swagger documentation for components API.
 *
 * @since 3.24
 */
@Tag(name = "components")
public interface ComponentsResourceDoc
{
  /**
   * @since 3.26
   */
  @Operation(summary = "List components")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "200", description = "Components returned",
          content = @Content(schema = @Schema(implementation = Page.class))),
      @ApiResponse(responseCode = "403", description = "Insufficient permissions to list components"),
      @ApiResponse(responseCode = "422", description = "Parameter 'repository' is required")
  })
  Page<ComponentXO> getComponents(
      @Parameter(
          description = "A token returned by a prior request. If present, the next page of results are returned") final String continuationToken,

      @Parameter(description = "Repository from which you would like to retrieve components",
          required = true) final String repository);

  /**
   * @since 3.26
   */
  @Operation(summary = "Get a single component")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "200", description = "Component returned",
          content = @Content(schema = @Schema(implementation = ComponentXO.class))),
      @ApiResponse(responseCode = "403", description = "Insufficient permissions to get component"),
      @ApiResponse(responseCode = "404", description = "Component not found"),
      @ApiResponse(responseCode = "422", description = "Malformed ID")
  })
  ComponentXO getComponentById(@Parameter(description = "ID of the component to retrieve") final String id);

  /**
   * @since 3.26
   */
  @Operation(summary = "Delete a single component")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "204", description = "Component was successfully deleted"),
      @ApiResponse(responseCode = "403", description = "Insufficient permissions to delete component"),
      @ApiResponse(responseCode = "404", description = "Component not found"),
      @ApiResponse(responseCode = "422", description = "Malformed ID")
  })
  void deleteComponent(@Parameter(description = "ID of the component to delete") final String id);

  @Operation(summary = "Upload a single component")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "403", description = "Insufficient permissions to upload a component"),
      @ApiResponse(responseCode = "422", description = "Parameter 'repository' is required")
  })
  void uploadComponent(
      @Parameter(description = "Name of the repository to which you would like to upload the component",
          required = true) final String repository,
      @Parameter(hidden = true) @MultipartForm HttpServletRequest request) throws IOException;
}
