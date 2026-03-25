
package org.globalart.astrum.repository.maven.rest;

import jakarta.ws.rs.BeanParam;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.POST;
import jakarta.ws.rs.PUT;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.PathParam;
import jakarta.ws.rs.core.Response;

import org.globalart.astrum.repository.rest.api.AbstractProxyRepositoriesApiResource;
import org.globalart.astrum.repository.rest.api.FormatAndType;
import org.globalart.astrum.repository.rest.api.model.AbstractApiRepository;

import io.swagger.v3.oas.annotations.tags.Tag;
import io.swagger.v3.oas.annotations.Operation;
import io.swagger.v3.oas.annotations.Parameter;
import io.swagger.v3.oas.annotations.responses.ApiResponse;
import io.swagger.v3.oas.annotations.responses.ApiResponses;

import static org.globalart.astrum.rest.ApiDocConstants.API_REPOSITORY_MANAGEMENT;
import static org.globalart.astrum.rest.ApiDocConstants.AUTHENTICATION_REQUIRED;
import static org.globalart.astrum.rest.ApiDocConstants.BAD_REQUEST;
import static org.globalart.astrum.rest.ApiDocConstants.INSUFFICIENT_PERMISSIONS;
import static org.globalart.astrum.rest.ApiDocConstants.REPOSITORY_CREATED;
import static org.globalart.astrum.rest.ApiDocConstants.REPOSITORY_UPDATED;

/**
 * @since 3.20
 */
@Tag(name = API_REPOSITORY_MANAGEMENT)
public abstract class MavenProxyRepositoriesApiResource
    extends AbstractProxyRepositoriesApiResource<MavenProxyRepositoryApiRequest>
{

  @Operation(summary = "Create Maven proxy repository")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "201", description = REPOSITORY_CREATED),
      @ApiResponse(responseCode = "401", description = AUTHENTICATION_REQUIRED),
      @ApiResponse(responseCode = "403", description = INSUFFICIENT_PERMISSIONS)
  })
  @POST
  @Override
  public Response createRepository(final MavenProxyRepositoryApiRequest request) {
    return super.createRepository(request);
  }

  @Operation(summary = "Update Maven proxy repository")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "204", description = REPOSITORY_UPDATED),
      @ApiResponse(responseCode = "400", description = BAD_REQUEST),
      @ApiResponse(responseCode = "401", description = AUTHENTICATION_REQUIRED),
      @ApiResponse(responseCode = "403", description = INSUFFICIENT_PERMISSIONS)
  })
  @PUT
  @Path("/{repositoryName}")
  @Override
  public Response updateRepository(
      final MavenProxyRepositoryApiRequest request,
      @Parameter(
          description = "Name of the repository to update") @PathParam("repositoryName") final String repositoryName)
  {
    return super.updateRepository(request, repositoryName);
  }

  @GET
  @Path("/{repositoryName}")
  @Operation(summary = "Get repository")
  @Override
  public AbstractApiRepository getRepository(
      @Parameter(hidden = true) @BeanParam final FormatAndType formatAndType,
      @PathParam("repositoryName") final String repositoryName)
  {
    return super.getRepository(formatAndType, repositoryName);
  }
}
