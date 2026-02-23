
package org.globalart.astrum.repository.rest.internal.resources.doc;

import java.util.List;

import org.globalart.astrum.repository.rest.api.RepositoryXO;
import org.globalart.astrum.repository.rest.internal.resources.RepositoriesResource;

import io.swagger.v3.oas.annotations.tags.Tag;
import io.swagger.v3.oas.annotations.Operation;
import io.swagger.v3.oas.annotations.Parameter;
import io.swagger.v3.oas.annotations.media.Content;
import io.swagger.v3.oas.annotations.media.Schema;
import io.swagger.v3.oas.annotations.responses.ApiResponse;
import io.swagger.v3.oas.annotations.responses.ApiResponses;

import static org.globalart.astrum.rest.ApiDocConstants.API_REPOSITORY_MANAGEMENT;
import static org.globalart.astrum.rest.ApiDocConstants.AUTHENTICATION_REQUIRED;
import static org.globalart.astrum.rest.ApiDocConstants.INSUFFICIENT_PERMISSIONS;
import static org.globalart.astrum.rest.ApiDocConstants.REPOSITORY_NOT_FOUND;

/**
 * Swagger documentation for {@link RepositoriesResource}
 *
 * @since 3.9
 */
@Tag(name = API_REPOSITORY_MANAGEMENT)
public interface RepositoriesResourceDoc
{
  @Operation(summary = "List repositories")
  List<RepositoryXO> getRepositories();

  @Operation(summary = "Get repository details")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "200", description = "Repository returned",
          content = @Content(schema = @Schema(implementation = RepositoryXO.class))),
      @ApiResponse(responseCode = "401", description = AUTHENTICATION_REQUIRED),
      @ApiResponse(responseCode = "403", description = INSUFFICIENT_PERMISSIONS),
      @ApiResponse(responseCode = "404", description = REPOSITORY_NOT_FOUND)
  })
  RepositoryXO getRepository(@Parameter(description = "Name of the repository to get") final String repositoryName);
}
