
package org.globalart.astrum.scheduling.internal.resources;

import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.concurrent.Future;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;
import jakarta.validation.ValidationException;
import jakarta.ws.rs.Consumes;
import jakarta.ws.rs.DELETE;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.NotAllowedException;
import jakarta.ws.rs.NotFoundException;
import jakarta.ws.rs.POST;
import jakarta.ws.rs.PUT;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.PathParam;
import jakarta.ws.rs.Produces;
import jakarta.ws.rs.QueryParam;
import jakarta.ws.rs.WebApplicationException;
import jakarta.ws.rs.core.Response;

import org.globalart.astrum.common.text.Strings2;
import org.globalart.astrum.rest.Page;
import org.globalart.astrum.rest.Resource;
import org.globalart.astrum.rest.ValidationErrorsException;
import org.globalart.astrum.rest.WebApplicationMessageException;
import org.globalart.astrum.scheduling.TaskConfiguration;
import org.globalart.astrum.scheduling.TaskDescriptor;
import org.globalart.astrum.scheduling.TaskInfo;
import org.globalart.astrum.scheduling.TaskNotificationCondition;
import org.globalart.astrum.scheduling.TaskScheduler;
import org.globalart.astrum.scheduling.TaskUtils;
import org.globalart.astrum.scheduling.api.TaskTemplateXO;
import org.globalart.astrum.scheduling.api.TaskXO;
import org.globalart.astrum.scheduling.internal.resources.doc.TasksApiResourceDoc;
import org.globalart.astrum.scheduling.schedule.Schedule;

import org.apache.shiro.authz.annotation.RequiresAuthentication;
import org.apache.shiro.authz.annotation.RequiresPermissions;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.lang.String.format;
import static java.util.Optional.ofNullable;
import static java.util.stream.Collectors.toList;
import static jakarta.ws.rs.core.MediaType.APPLICATION_JSON;
import static jakarta.ws.rs.core.Response.Status.CONFLICT;
import static jakarta.ws.rs.core.Response.Status.CREATED;
import static jakarta.ws.rs.core.Response.Status.INTERNAL_SERVER_ERROR;
import static org.globalart.astrum.rest.APIConstants.V1_API_PREFIX;
import org.springframework.stereotype.Component;

/**
 * @since 3.6
 */
@Component
@Path(TasksApiResource.RESOURCE_URI)
@Produces(APPLICATION_JSON)
public class TasksApiResource
    implements Resource, TasksApiResourceDoc
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  public static final String RESOURCE_URI = V1_API_PREFIX + "/tasks";

  private static final String TRIGGER_SOURCE = "REST API";

  private static final String SCRIPT_TYPE_ID = "script";

  private static final String SCRIPT_SOURCE = "source";

  private static final String INVALID = "Invalid task";

  private final TaskScheduler taskScheduler;

  private final TaskUtils taskUtils;

  private final boolean allowScriptCreation;

  @Autowired
  public TasksApiResource(
      final TaskScheduler taskScheduler,
      final TaskUtils taskUtils,
      @Value("${astrum.scripts.allowCreation:false}") final boolean allowScriptCreation)
  {
    this.taskScheduler = checkNotNull(taskScheduler);
    this.taskUtils = checkNotNull(taskUtils);
    this.allowScriptCreation = allowScriptCreation;
  }

  @Override
  @GET
  @RequiresAuthentication
  @RequiresPermissions("astrum:tasks:read")
  public Page<TaskXO> getTasks(@QueryParam("type") final String type) {
    List<TaskXO> taskXOs = taskScheduler.listsTasks()
        .stream()
        .filter(taskInfo -> taskInfo.getConfiguration().isVisible())
        .filter(taskInfo -> typeParameterMatches(type, taskInfo))
        .map(taskInfo -> TaskXO.fromTaskInfo(taskInfo, taskScheduler.toExternalTaskState(taskInfo)))
        .collect(toList());

    return new Page<>(taskXOs, null);
  }

  @Override
  @GET
  @Path("/{id}")
  @RequiresAuthentication
  @RequiresPermissions("astrum:tasks:read")
  public TaskXO getTaskById(@PathParam("id") final String id) {
    TaskInfo task = getTaskInfo(id);
    return TaskXO.fromTaskInfo(task, taskScheduler.toExternalTaskState(task));
  }

  @Override
  @POST
  @Path("/{id}/run")
  @RequiresAuthentication
  @RequiresPermissions("astrum:tasks:start")
  public void run(@PathParam("id") final String id) {
    try {
      TaskInfo taskInfo = getTaskInfo(id);

      if (!taskInfo.getConfiguration().isEnabled()) {
        throw new NotAllowedException(format("Task %s is disabled", id));
      }

      taskInfo.runNow(TRIGGER_SOURCE);
    }
    catch (NotFoundException | NotAllowedException e) {
      throw e;
    }
    catch (Exception e) {
      log.error("error running task with id {}", id, e);
      throw new WebApplicationException(format("Error running task %s", id), INTERNAL_SERVER_ERROR);
    }
  }

  @Override
  @POST
  @Path("/{id}/stop")
  @RequiresAuthentication
  @RequiresPermissions("astrum:tasks:stop")
  public void stop(@PathParam("id") final String id) {
    try {
      TaskInfo taskInfo = getTaskInfo(id);
      Future<?> taskFuture = taskInfo.getCurrentState().getFuture();
      if (taskFuture == null) {
        throw new WebApplicationException(format("Task %s is not running", id), CONFLICT);
      }
      if (!taskFuture.cancel(false)) {
        throw new WebApplicationException(format("Unable to stop task %s", id), CONFLICT);
      }
    }
    catch (WebApplicationException webApplicationException) {
      throw webApplicationException;
    }
    catch (Exception e) {
      log.error("error stopping task with id {}", id, e);
      throw new WebApplicationException(format("Error running task %s", id), INTERNAL_SERVER_ERROR);
    }
  }

  @Override
  @GET
  @Path("/templates")
  @RequiresAuthentication
  @RequiresPermissions("astrum:tasks:read")
  public List<TaskTemplateXO> getTemplates() {
    return taskScheduler.getTaskFactory()
        .getDescriptors()
        .stream()
        .filter(TaskDescriptor::isExposed)
        .map(TaskTemplateXO::fromDescriptor)
        .collect(toList());
  }

  @Override
  @GET
  @Path("/templates/{typeId}")
  @RequiresAuthentication
  @RequiresPermissions("astrum:tasks:read")
  public TaskTemplateXO getTemplate(@PathParam("typeId") final String typeId) {
    return TaskTemplateXO.fromDescriptor(getExposedDescriptor(typeId));
  }

  @Override
  @POST
  @Consumes(APPLICATION_JSON)
  @RequiresAuthentication
  @RequiresPermissions("astrum:tasks:create")
  public Response createTask(final TaskTemplateXO template) {
    if (template == null) {
      throw new ValidationErrorsException("A task is required");
    }
    if (Strings2.isBlank(template.getType())) {
      throw new ValidationErrorsException("type", "Task type is required");
    }
    TaskDescriptor descriptor = taskScheduler.getTaskFactory().findDescriptor(template.getType());
    if (descriptor == null || !descriptor.isExposed()) {
      throw new ValidationErrorsException("type", "Unknown task type: " + template.getType());
    }
    if (Strings2.isBlank(template.getName())) {
      throw new ValidationErrorsException("name", "Name is required");
    }

    Schedule schedule =
        new TaskScheduleConverter(taskScheduler.getScheduleFactory()).asSchedule(template.getFrequency());
    Map<String, String> properties = template.getProperties() != null ? template.getProperties() : Map.of();
    taskUtils.validateTaskCreationForAPI(template.getName(), descriptor.getId(), properties, schedule);

    TaskConfiguration configuration = taskScheduler.createTaskConfigurationInstance(descriptor.getId());
    applyProperties(configuration, properties);
    configuration.setTypeId(descriptor.getId());
    configuration.setName(template.getName().trim());
    configuration.setEnabled(template.getEnabled() == null || template.getEnabled());
    configuration.setAlertEmail(Strings2.isBlank(template.getAlertEmail()) ? null : template.getAlertEmail().trim());
    configuration.setNotificationCondition(template.getNotificationCondition() != null
        ? template.getNotificationCondition()
        : TaskNotificationCondition.DEFAULT);

    TaskInfo task = schedule(configuration, schedule);
    return Response.status(CREATED).entity(TaskXO.fromTaskInfo(task, taskScheduler.toExternalTaskState(task))).build();
  }

  @Override
  @PUT
  @Path("/{id}")
  @Consumes(APPLICATION_JSON)
  @RequiresAuthentication
  @RequiresPermissions("astrum:tasks:update")
  public TaskXO updateTask(@PathParam("id") final String id, final TaskTemplateXO template) {
    TaskInfo task = getTaskInfo(id);
    if (template == null) {
      throw new ValidationErrorsException("A task is required");
    }
    if (template.getType() != null && !Objects.equals(template.getType(), task.getTypeId())) {
      throw new ValidationErrorsException("type", "Task type cannot be changed");
    }
    if (template.getName() != null && Strings2.isBlank(template.getName())) {
      throw new ValidationErrorsException("name", "Name is required");
    }
    if (taskScheduler.toExternalTaskState(task).getState().isRunning()) {
      throw new WebApplicationMessageException(CONFLICT,
          "Task can not be edited while it is being executed or it is in line to be executed", APPLICATION_JSON);
    }
    Map<String, String> properties = template.getProperties() != null ? template.getProperties() : Map.of();
    if (SCRIPT_TYPE_ID.equals(task.getTypeId()) && !allowScriptCreation && properties.containsKey(SCRIPT_SOURCE)
        && !Objects.equals(task.getConfiguration().getString(SCRIPT_SOURCE), properties.get(SCRIPT_SOURCE))) {
      throw new ValidationErrorsException("properties." + SCRIPT_SOURCE, "Script source updates are not allowed");
    }

    Schedule schedule = template.getFrequency() != null
        ? new TaskScheduleConverter(taskScheduler.getScheduleFactory()).asSchedule(template.getFrequency())
        : task.getSchedule();

    TaskConfiguration configuration = taskScheduler.createTaskConfigurationInstance(task.getTypeId());
    configuration.apply(task.getConfiguration());
    applyProperties(configuration, properties);
    configuration.setTypeId(task.getTypeId());
    if (template.getName() != null) {
      configuration.setName(template.getName().trim());
    }
    if (template.getEnabled() != null) {
      configuration.setEnabled(template.getEnabled());
    }
    if (template.getAlertEmail() != null) {
      configuration.setAlertEmail(Strings2.isBlank(template.getAlertEmail()) ? null : template.getAlertEmail().trim());
    }
    if (template.getNotificationCondition() != null) {
      configuration.setNotificationCondition(template.getNotificationCondition());
    }

    TaskInfo updated = schedule(configuration, schedule);
    return TaskXO.fromTaskInfo(updated, taskScheduler.toExternalTaskState(updated));
  }

  @Override
  @DELETE
  @Path("/{id}")
  @RequiresAuthentication
  @RequiresPermissions("astrum:tasks:delete")
  public void deleteTask(@PathParam("id") final String id) {
    getTaskInfo(id).remove();
  }

  /**
   * Copies the properties onto the configuration, leaving out the internal ones (their keys start with a dot) so a
   * client cannot change what the scheduler keeps about a task.
   */
  private static void applyProperties(final TaskConfiguration configuration, final Map<String, String> properties) {
    properties.entrySet()
        .stream()
        .filter(entry -> entry.getKey() != null && !entry.getKey().startsWith("."))
        .forEach(entry -> configuration.setString(entry.getKey(), entry.getValue()));
  }

  /**
   * Schedules the task, answering a configuration the task type rejects with a validation error.
   */
  private TaskInfo schedule(final TaskConfiguration configuration, final Schedule schedule) {
    try {
      return taskScheduler.scheduleTask(configuration, schedule);
    }
    catch (ValidationErrorsException e) {
      throw e;
    }
    catch (ValidationException | IllegalArgumentException | IllegalStateException e) {
      log.debug("Rejected task {} of type {}", configuration.getName(), configuration.getTypeId(), e);
      throw new ValidationErrorsException(e.getMessage() != null ? e.getMessage() : INVALID, e);
    }
  }

  private TaskDescriptor getExposedDescriptor(final String typeId) {
    TaskDescriptor descriptor = taskScheduler.getTaskFactory().findDescriptor(typeId);
    if (descriptor == null || !descriptor.isExposed()) {
      throw new NotFoundException("Unable to locate task type " + typeId);
    }
    return descriptor;
  }

  private TaskInfo getTaskInfo(final String id) {
    return ofNullable(taskScheduler.getTaskById(id))
        .filter(taskInfo -> taskInfo.getConfiguration().isVisible())
        .orElseThrow(() -> new NotFoundException("Unable to locate task with id " + id));
  }

  private static boolean typeParameterMatches(final String type, final TaskInfo taskInfo) {
    return type == null || type.isEmpty() || type.equals(taskInfo.getTypeId());
  }
}
