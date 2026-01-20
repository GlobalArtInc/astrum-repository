
package org.globalart.astrum.api.rest.common.status;

import jakarta.ws.rs.GET;
import jakarta.ws.rs.core.Response;

import io.swagger.v3.oas.annotations.tags.Tag;
import io.swagger.v3.oas.annotations.Operation;
import io.swagger.v3.oas.annotations.responses.ApiResponse;
import io.swagger.v3.oas.annotations.responses.ApiResponses;

/**
 * REST API for status operations
 *
 * @since 3.15
 */
@Tag(name = "Status")
public interface StatusResourceDoc
{
  /**
   * @return 200 if the server is available to serve read requests, 503 otherwise
   */
  @GET
  @Operation(summary = "Health check endpoint that validates server can respond to read requests")
  @ApiResponses({
      @ApiResponse(responseCode = "200", description = "Available to service requests"),
      @ApiResponse(responseCode = "503", description = "Unavailable to service requests")
  })
  Response isAvailable();

  /**
   * @return 200 if the server is available to serve read and write requests, 503 otherwise
   *
   * @since 3.16
   */
  @GET
  @Operation(summary = "Health check endpoint that validates server can respond to read and write requests")
  @ApiResponses({
      @ApiResponse(responseCode = "200", description = "Available to service requests"),
      @ApiResponse(responseCode = "503", description = "Unavailable to service requests")
  })
  Response isWritable();
}
