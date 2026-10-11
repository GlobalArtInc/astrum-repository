
package org.globalart.astrum.scheduling.internal.resources.doc;

import io.swagger.v3.oas.annotations.tags.Tag;
import io.swagger.v3.oas.annotations.Operation;
import io.swagger.v3.oas.annotations.Parameter;
import io.swagger.v3.oas.annotations.media.Content;
import io.swagger.v3.oas.annotations.media.Schema;
import io.swagger.v3.oas.annotations.responses.ApiResponse;
import io.swagger.v3.oas.annotations.responses.ApiResponses;

import java.util.List;

import jakarta.ws.rs.core.Response;

import org.globalart.astrum.scheduling.api.TaskTemplateXO;
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

  @Operation(summary = "List the templates of the task types that can be created")
  List<TaskTemplateXO> getTemplates();

  @Operation(summary = "Get the template of a task type")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "200", description = "Template returned",
          content = @Content(schema = @Schema(implementation = TaskTemplateXO.class))),
      @ApiResponse(responseCode = "404", description = "Task type not found or not available")
  })
  TaskTemplateXO getTemplate(@Parameter(description = "Id of the task type") final String typeId);

  @Operation(summary = "Create a task")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "201", description = "Task created",
          content = @Content(schema = @Schema(implementation = TaskXO.class))),
      @ApiResponse(responseCode = "400", description = "Invalid task"),
      @ApiResponse(responseCode = "409", description = "A task with the same name or configuration exists")
  })
  Response createTask(final TaskTemplateXO template);

  @Operation(summary = "Update a task; properties that are sent replace the stored ones, others are kept")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "200", description = "Task updated",
          content = @Content(schema = @Schema(implementation = TaskXO.class))),
      @ApiResponse(responseCode = "400", description = "Invalid task"),
      @ApiResponse(responseCode = "404", description = "Task not found"),
      @ApiResponse(responseCode = "409", description = "Task is running")
  })
  TaskXO updateTask(@Parameter(description = "Id of the task to update") final String id,
                    final TaskTemplateXO template);

  @Operation(summary = "Delete a task")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "204", description = "Task deleted"),
      @ApiResponse(responseCode = "404", description = "Task not found")
  })
  void deleteTask(@Parameter(description = "Id of the task to delete") final String id);
}
