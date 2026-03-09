
package org.globalart.astrum.script;

import jakarta.validation.Valid;
import jakarta.validation.constraints.NotNull;
import jakarta.ws.rs.*;
import jakarta.ws.rs.core.MediaType;
import java.util.List;

/**
 * Public API for managing Scripts. Provides BREAD capabilities.
 *
 * @since 3.0
 */
@Path(ScriptClient.RESOURCE_URI)
@Produces(MediaType.APPLICATION_JSON)
@Consumes(MediaType.APPLICATION_JSON)
public interface ScriptClient
{

  String RESOURCE_URI = "/v1/script";

  String RUN_ACTION = "run";

  /**
   * Browse all {@link Script}.
   */
  @GET
  List<ScriptXO> browse();

  /**
   * Get a specific {@link Script} by name.
   */
  @GET
  @Path("{name}")
  ScriptXO read(@PathParam("name") String name);

  /**
   * Edit an existing {@link Script}.
   */
  @PUT
  @Path("{name}")
  void edit(@PathParam("name") String name, @NotNull @Valid ScriptXO scriptXO);

  /**
   * Add a new {@link Script}.
   */
  @POST
  void add(@NotNull @Valid ScriptXO scriptXO);

  /**
   * Delete an existing {@link Script}.
   */
  @DELETE
  @Path("{name}")
  void delete(@PathParam("name") String name);

  /**
   * Run an existing {@link Script}.
   * 
   * @param name the name of the Script to execute
   * @param args the arguments to pass to the Script
   */
  @POST
  @Path("{name}/run")
  @Consumes({MediaType.TEXT_PLAIN, MediaType.APPLICATION_JSON})
  ScriptResultXO run(@PathParam("name") String name, String args);
}
