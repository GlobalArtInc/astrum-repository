
package org.globalart.astrum.api.rest.selfhosted.blobstore.file;

import jakarta.validation.Valid;

import org.globalart.astrum.api.rest.common.blobstore.file.model.FileBlobStoreApiCreateRequest;
import org.globalart.astrum.api.rest.common.blobstore.file.model.FileBlobStoreApiModel;
import org.globalart.astrum.api.rest.common.blobstore.file.model.FileBlobStoreApiUpdateRequest;
import org.globalart.astrum.validation.Validate;

import io.swagger.v3.oas.annotations.tags.Tag;
import io.swagger.v3.oas.annotations.Operation;
import io.swagger.v3.oas.annotations.Parameter;
import io.swagger.v3.oas.annotations.responses.ApiResponse;
import io.swagger.v3.oas.annotations.responses.ApiResponses;
import io.swagger.v3.oas.annotations.media.Content;
import io.swagger.v3.oas.annotations.media.Schema;

import static org.globalart.astrum.rest.ApiDocConstants.BLOBSTORE_NOT_FOUND;
import static org.globalart.astrum.rest.ApiDocConstants.INSUFFICIENT_PERMISSIONS;
import static org.globalart.astrum.rest.ApiDocConstants.SUCCESS;

/**
 * REST facade for {@link FileBlobStoreResource}
 *
 * @since 3.19
 */
@Tag(name = "Blob store")
public interface FileBlobStoreResourceDoc
{
  @Operation(summary = "Create a file blob store")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "204", description = SUCCESS),
      @ApiResponse(responseCode = "403", description = INSUFFICIENT_PERMISSIONS)
  })
  void createFileBlobStore(@Valid final FileBlobStoreApiCreateRequest request) throws Exception;

  @Operation(summary = "Update a file blob store configuration by name")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "204", description = SUCCESS),
      @ApiResponse(responseCode = "403", description = INSUFFICIENT_PERMISSIONS),
      @ApiResponse(responseCode = "404", description = BLOBSTORE_NOT_FOUND)
  })
  @Validate
  void updateFileBlobStore(
      @Parameter(description = "The name of the file blob store to update") final String name,
      @Valid final FileBlobStoreApiUpdateRequest request) throws Exception;

  @Operation(summary = "Get a file blob store configuration by name")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "200", description = SUCCESS,
          content = @Content(schema = @Schema(implementation = FileBlobStoreApiModel.class))),
      @ApiResponse(responseCode = "403", description = INSUFFICIENT_PERMISSIONS),
      @ApiResponse(responseCode = "404", description = BLOBSTORE_NOT_FOUND)
  })
  FileBlobStoreApiModel getFileBlobStoreConfiguration(
      @Parameter(description = "The name of the file blob store to read", example = "default") final String name);
}
