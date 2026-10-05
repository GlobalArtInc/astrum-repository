
package org.globalart.astrum.api.rest.selfhosted.security.usersource;

import java.util.List;

import org.globalart.astrum.api.rest.selfhosted.security.usersource.model.ApiUserSource;
import org.globalart.astrum.security.internal.rest.AstrumSecurityApiConstants;

import io.swagger.v3.oas.annotations.tags.Tag;
import io.swagger.v3.oas.annotations.Operation;
import io.swagger.v3.oas.annotations.media.ArraySchema;
import io.swagger.v3.oas.annotations.media.Content;
import io.swagger.v3.oas.annotations.media.Schema;
import io.swagger.v3.oas.annotations.responses.ApiResponse;
import io.swagger.v3.oas.annotations.responses.ApiResponses;

/**
 * Swagger documentation for {@link SecurityApiResource}
 *
 * @since 3.17
 */
@Tag(name = "Security management")
public interface SecurityApiResourceDoc
{
  @Operation(summary = "Retrieve a list of the available user sources.")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "200", description = "User sources returned",
          content = @Content(array = @ArraySchema(schema = @Schema(implementation = ApiUserSource.class)))),
      @ApiResponse(responseCode = "403", description = AstrumSecurityApiConstants.INVALID_PERMISSIONS)
  })
  List<ApiUserSource> getUserSources();
}
