
package org.globalart.astrum.repository.rest.api;

import jakarta.ws.rs.core.Response;

import io.swagger.v3.oas.annotations.tags.Tag;
import io.swagger.v3.oas.annotations.Operation;
import io.swagger.v3.oas.annotations.Parameter;
import io.swagger.v3.oas.annotations.responses.ApiResponse;
import io.swagger.v3.oas.annotations.responses.ApiResponses;

import static org.globalart.astrum.rest.ApiDocConstants.API_REPOSITORY_MANAGEMENT;
import static org.globalart.astrum.rest.ApiDocConstants.AUTHENTICATION_REQUIRED;
import static org.globalart.astrum.rest.ApiDocConstants.INSUFFICIENT_PERMISSIONS;
import static org.globalart.astrum.rest.ApiDocConstants.REPOSITORY_DELETED;
import static org.globalart.astrum.rest.ApiDocConstants.REPOSITORY_NOT_FOUND;

/**
 * @since 3.20
 */
@Tag(name = API_REPOSITORY_MANAGEMENT)
public interface RepositoriesApiResourceDoc
{
  @Operation(summary = "Delete repository of any format")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "204", description = REPOSITORY_DELETED),
      @ApiResponse(responseCode = "401", description = AUTHENTICATION_REQUIRED),
      @ApiResponse(responseCode = "403", description = INSUFFICIENT_PERMISSIONS),
      @ApiResponse(responseCode = "404", description = REPOSITORY_NOT_FOUND)
  })
  Response deleteRepository(
      @Parameter(description = "Name of the repository to delete") final String repositoryName) throws Exception;

  @Operation(summary = "Schedule a 'Repair - Rebuild repository search' Task. Hosted or proxy repositories only.")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "204", description = "Repository search index rebuild has been scheduled"),
      @ApiResponse(responseCode = "400", description = "Repository is not of hosted or proxy type"),
      @ApiResponse(responseCode = "401", description = AUTHENTICATION_REQUIRED),
      @ApiResponse(responseCode = "403", description = INSUFFICIENT_PERMISSIONS),
      @ApiResponse(responseCode = "404", description = REPOSITORY_NOT_FOUND)
  })
  void rebuildIndex(@Parameter(description = "Name of the repository to rebuild index") final String repositoryName);

  @Operation(summary = "Invalidate repository cache. Proxy or group repositories only.")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "204", description = "Repository cache invalidated"),
      @ApiResponse(responseCode = "400", description = "Repository is not of proxy or group type"),
      @ApiResponse(responseCode = "401", description = AUTHENTICATION_REQUIRED),
      @ApiResponse(responseCode = "403", description = INSUFFICIENT_PERMISSIONS),
      @ApiResponse(responseCode = "404", description = REPOSITORY_NOT_FOUND)
  })
  void invalidateCache(
      @Parameter(description = "Name of the repository to invalidate cache") final String repositoryName);
}
