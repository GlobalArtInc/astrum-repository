
package org.globalart.astrum.security.internal.rest;

import java.util.Collection;

import io.swagger.v3.oas.annotations.tags.Tag;
import io.swagger.v3.oas.annotations.Operation;
import io.swagger.v3.oas.annotations.Parameter;
import io.swagger.v3.oas.annotations.media.ArraySchema;
import io.swagger.v3.oas.annotations.media.Content;
import io.swagger.v3.oas.annotations.media.Schema;
import io.swagger.v3.oas.annotations.responses.ApiResponse;
import io.swagger.v3.oas.annotations.responses.ApiResponses;

/**
 * Swagger documentation for {@link UserApiResource}
 *
 * @since 3.17
 */
@Tag(name = "Security management: users")
public interface UserApiResourceDoc
{
  String USER_ID_DESCRIPTION = "The userid the request should apply to.";

  String REALM_DESCRIPTION = "The realm the request should apply to.";

  String PASSWORD_REQUIRED = "Password was not supplied in the body of the request";

  @Operation(summary = "Retrieve a list of users. For SAML user sources a limit of 1000 users will be applied.")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "200", description = "Users returned",
          content = @Content(array = @ArraySchema(schema = @Schema(implementation = ApiUser.class)))),
      @ApiResponse(responseCode = "400", description = PASSWORD_REQUIRED),
      @ApiResponse(responseCode = "403", description = AstrumSecurityApiConstants.INVALID_PERMISSIONS)})
  Collection<ApiUser> getUsers(
      @Parameter(description = "An optional term to search userids for.") String userId,
      @Parameter(description = "An optional user source to restrict the search to.") String source);

  @Operation(summary = "Delete a user.")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "400", description = AstrumSecurityApiConstants.NON_LOCAL_USER_CANNOT_BE_DELETED),
      @ApiResponse(responseCode = "403", description = AstrumSecurityApiConstants.INVALID_PERMISSIONS),
      @ApiResponse(responseCode = "404", description = AstrumSecurityApiConstants.USER_OR_SOURCE_NOT_FOUND)})
  void deleteUser(
      @Parameter(description = USER_ID_DESCRIPTION) String userId,
      @Parameter(description = REALM_DESCRIPTION) String realm);
}
