
package org.globalart.astrum.api.rest.selfhosted.node;

import org.globalart.astrum.api.rest.selfhosted.node.NodeIdApiResource.NodeInformation;

import io.swagger.v3.oas.annotations.tags.Tag;
import io.swagger.v3.oas.annotations.Operation;
import io.swagger.v3.oas.annotations.media.Content;
import io.swagger.v3.oas.annotations.media.Schema;
import io.swagger.v3.oas.annotations.responses.ApiResponse;
import io.swagger.v3.oas.annotations.responses.ApiResponses;

/**
 * REST API to reset the stored Node ID. This is intended for use when cloning a system.
 *
 * @since 3.37
 */
@Tag(name = "System: Nodes")
public interface NodeIdApiResourceDoc
{
  @Operation(summary = "Get information about this node")
  @ApiResponses(
      value = {
          @ApiResponse(responseCode = "200", description = "Node information returned",
              content = @Content(schema = @Schema(implementation = NodeInformation.class))),
          @ApiResponse(responseCode = "403", description = "Insufficient permissions to update settings")})
  NodeInformation getNodeId();

  @Operation(
      summary = "Reset the ID for this node. Takes effect after restart and should only be used when cloning an instance")
  @ApiResponses(
      value = {@ApiResponse(responseCode = "403", description = "Insufficient permissions to update settings")})
  void clear();
}
