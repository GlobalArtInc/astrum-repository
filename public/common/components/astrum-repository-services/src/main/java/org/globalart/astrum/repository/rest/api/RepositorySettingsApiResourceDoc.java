
package org.globalart.astrum.repository.rest.api;

import java.util.List;

import org.globalart.astrum.repository.rest.api.model.AbstractApiRepository;

import io.swagger.v3.oas.annotations.tags.Tag;
import io.swagger.v3.oas.annotations.Operation;
import io.swagger.v3.oas.annotations.responses.ApiResponse;
import io.swagger.v3.oas.annotations.responses.ApiResponses;
import io.swagger.v3.oas.annotations.media.Content;
import io.swagger.v3.oas.annotations.media.Schema;
import io.swagger.v3.oas.annotations.media.ArraySchema;

import static org.globalart.astrum.rest.ApiDocConstants.API_REPOSITORY_MANAGEMENT;
import static org.globalart.astrum.rest.ApiDocConstants.AUTHENTICATION_REQUIRED;
import static org.globalart.astrum.rest.ApiDocConstants.INSUFFICIENT_PERMISSIONS;

/**
 * @since 3.26
 */
@Tag(name = API_REPOSITORY_MANAGEMENT)
public interface RepositorySettingsApiResourceDoc
{
  @Operation(summary = "List repositories")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "200", description = "Repositories list returned",
          content = @Content(array = @ArraySchema(schema = @Schema(implementation = AbstractApiRepository.class)))),
      @ApiResponse(responseCode = "401", description = AUTHENTICATION_REQUIRED),
      @ApiResponse(responseCode = "403", description = INSUFFICIENT_PERMISSIONS)
  })
  List<AbstractApiRepository> getRepositories();
}
