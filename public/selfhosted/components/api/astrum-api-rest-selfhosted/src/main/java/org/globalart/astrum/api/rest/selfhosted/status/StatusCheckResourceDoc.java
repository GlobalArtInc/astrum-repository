
package org.globalart.astrum.api.rest.selfhosted.status;

import jakarta.ws.rs.GET;

import com.codahale.metrics.health.HealthCheck.Result;
import io.swagger.v3.oas.annotations.tags.Tag;
import io.swagger.v3.oas.annotations.Operation;
import io.swagger.v3.oas.annotations.media.Content;
import io.swagger.v3.oas.annotations.media.Schema;
import io.swagger.v3.oas.annotations.responses.ApiResponse;
import io.swagger.v3.oas.annotations.responses.ApiResponses;
import java.util.SortedMap;

@Tag(name = "Status")
public interface StatusCheckResourceDoc
{
  @GET
  @Operation(summary = "Health check endpoint that returns the results of the system status checks")
  @ApiResponses({
      @ApiResponse(responseCode = "200", description = "The system status check results",
          content = @Content(schema = @Schema(type = "object")))
  })
  SortedMap<String, Result> getSystemStatusChecks();
}
