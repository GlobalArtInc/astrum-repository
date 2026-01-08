
package org.globalart.astrum.scheduling.internal.resources.doc;

import io.swagger.v3.oas.annotations.tags.Tag;
import io.swagger.v3.oas.annotations.Operation;
import io.swagger.v3.oas.annotations.Parameter;
import io.swagger.v3.oas.annotations.media.Content;
import io.swagger.v3.oas.annotations.media.Schema;
import io.swagger.v3.oas.annotations.responses.ApiResponse;
import io.swagger.v3.oas.annotations.responses.ApiResponses;

import org.globalart.astrum.scheduling.api.TaskXO;
import org.globalart.astrum.rest.Page;

/**
 * Swagger documentation for {@link TasksResource}
 *
 * @since 3.6
 */
@Tag(name = "Tasks")
public interface TasksApiResourceDoc
{
  @Operation(summary = "List tasks")
  Page<TaskXO> getTasks(@Parameter(description = "Type of the tasks to get") final String type);

  @Operation(summary = "Get a single task by id")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "200", description = "Task returned",
          content = @Content(schema = @Schema(implementation = TaskXO.class))),
      @ApiResponse(responseCode = "404", description = "Task not found")
  })
  TaskXO getTaskById(@Parameter(description = "Id of the task to get") final String id);

  @Operation(summary = "Run task")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "204", description = "Task was run"),
      @ApiResponse(responseCode = "404", description = "Task not found"),
      @ApiResponse(responseCode = "405", description = "Task is disabled")
  })
  void run(@Parameter(description = "Id of the task to run") final String id);

  @Operation(summary = "Stop task")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "204", description = "Task was stopped"),
      @ApiResponse(responseCode = "409", description = "Unable to stop task"),
      @ApiResponse(responseCode = "404", description = "Task not found")
  })
  void stop(@Parameter(description = "Id of the task to stop") final String id);
}
