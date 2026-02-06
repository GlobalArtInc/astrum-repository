
package org.globalart.astrum.repository.content.rest.internal.resources.doc;

import org.globalart.astrum.repository.content.rest.internal.resources.AssetsResource;
import org.globalart.astrum.repository.rest.api.AssetXO;
import org.globalart.astrum.rest.Page;

import io.swagger.v3.oas.annotations.tags.Tag;
import io.swagger.v3.oas.annotations.Operation;
import io.swagger.v3.oas.annotations.Parameter;
import io.swagger.v3.oas.annotations.media.Content;
import io.swagger.v3.oas.annotations.media.Schema;
import io.swagger.v3.oas.annotations.responses.ApiResponse;
import io.swagger.v3.oas.annotations.responses.ApiResponses;

/**
 * Swagger documentation for {@link AssetsResource}
 *
 * @since 3.4
 */
@Tag(name = "assets", description = "Operations to get and delete assets")
public interface AssetsResourceDoc
{
  @Operation(summary = "List assets")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "200", description = "Assets returned",
          content = @Content(schema = @Schema(implementation = Page.class))),
      @ApiResponse(responseCode = "403", description = "Insufficient permissions to list assets"),
      @ApiResponse(responseCode = "422", description = "Parameter 'repository' is required")
  })
  Page<AssetXO> getAssets(
      @Parameter(
          description = "A token returned by a prior request. If present, the next page of results are returned") final String continuationToken,

      @Parameter(description = "Repository from which you would like to retrieve assets.",
          required = true) final String repository);

  @Operation(summary = "Get a single asset")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "200", description = "Asset returned",
          content = @Content(schema = @Schema(implementation = AssetXO.class))),
      @ApiResponse(responseCode = "403", description = "Insufficient permissions to get asset"),
      @ApiResponse(responseCode = "404", description = "Asset not found"),
      @ApiResponse(responseCode = "422", description = "Malformed ID")
  })
  AssetXO getAssetById(@Parameter(description = "Id of the asset to get") final String id);

  @Operation(summary = "Delete a single asset")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "204", description = "Asset was successfully deleted"),
      @ApiResponse(responseCode = "403", description = "Insufficient permissions to delete asset"),
      @ApiResponse(responseCode = "404", description = "Asset not found"),
      @ApiResponse(responseCode = "422", description = "Malformed ID")
  })
  void deleteAsset(@Parameter(description = "Id of the asset to delete") final String id);

}
