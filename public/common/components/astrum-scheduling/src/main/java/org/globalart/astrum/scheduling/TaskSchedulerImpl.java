
package org.globalart.astrum.scheduling;

import java.util.Date;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.UUID;
import java.util.stream.Collectors;

import javax.annotation.Nullable;
import org.springframework.beans.factory.annotation.Autowired;
import jakarta.inject.Provider;
import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.rest.ValidationErrorsException;
import org.globalart.astrum.scheduling.events.TaskScheduledEvent;
import org.globalart.astrum.scheduling.schedule.Schedule;
import org.globalart.astrum.scheduling.schedule.ScheduleFactory;
import org.globalart.astrum.scheduling.spi.SchedulerSPI;

import org.springframework.beans.factory.annotation.Value;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkArgument;
import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.base.Preconditions.checkState;
import static org.globalart.astrum.common.app.FeatureFlags.CHANGE_REPO_BLOBSTORE_TASK_ENABLED_NAMED_VALUE;

import org.springframework.context.annotation.Lazy;
import org.springframework.stereotype.Component;

/**
 * Default {@link TaskScheduler} implementation.
 *
 * @since 3.0
 */
@Component
public class TaskSchedulerImpl
    implements TaskScheduler
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  protected static final String REPO_MOVE_TYPE_ID = "repository.move";

  private final EventManager eventManager;

  private final TaskFactory taskFactory;

  private final Provider<SchedulerSPI> scheduler;

  protected final boolean changeRepoBlobstoreTaskEnabled;

  @Autowired
  public TaskSchedulerImpl(
      final EventManager eventManager,
      @Lazy final TaskFactory taskFactory,
      final Provider<SchedulerSPI> scheduler,
      @Value(CHANGE_REPO_BLOBSTORE_TASK_ENABLED_NAMED_VALUE) final boolean changeRepoBlobstoreTaskEnabled)
  {
    this.eventManager = checkNotNull(eventManager);
    this.taskFactory = checkNotNull(taskFactory);
    this.scheduler = checkNotNull(scheduler);
    this.changeRepoBlobstoreTaskEnabled = changeRepoBlobstoreTaskEnabled;
  }

  @Override
  public TaskFactory getTaskFactory() {
    return taskFactory;
  }

  /**
   * Helper to ensure provided scheduler is non-null.
   */
  private SchedulerSPI getScheduler() {
    SchedulerSPI result = scheduler.get();
    checkState(result != null);
    return result;
  }

  @Override
  public ScheduleFactory getScheduleFactory() {
    ScheduleFactory result = getScheduler().scheduleFactory();
    checkState(result != null);
    return result;
  }

  @Override
  public int getRunningTaskCount() {
    return getScheduler().getRunningTaskCount();
  }

  @Override
  public int getExecutedTaskCount() {
    return getScheduler().getExecutedTaskCount();
  }

  @Override
  public TaskConfiguration createTaskConfigurationInstance(final String typeId) {
    checkNotNull(typeId);

    TaskDescriptor descriptor = taskFactory.findDescriptor(typeId);
    checkArgument(descriptor != null, "Missing descriptor for task with type-id: %s", typeId);

    TaskConfiguration config = descriptor.createTaskConfiguration();
    descriptor.initializeConfiguration(config); // in case any hardcode values need to be inserted
    config.setId(UUID.randomUUID().toString());
    config.setTypeId(descriptor.getId());
    config.setTypeName(descriptor.getName());
    config.setName(descriptor.getName());
    config.setVisible(descriptor.isVisible());
    config.setRecoverable(descriptor.isRecoverable());
    config.setExposed(descriptor.isExposed());

    return config;
  }

  @Override
  public TaskInfo submit(final TaskConfiguration config) {
    return scheduleTask(config, getScheduleFactory().now());
  }

  @Override
  public TaskInfo getTaskById(final String id) {
    checkNotNull(id);
    TaskInfo taskInfo = getScheduler().getTaskById(id);
    if (null != taskInfo && includeRepoMoveTask(taskInfo)) {
      return taskInfo;
    }
    return null;
  }

  @Override
  public List<TaskInfo> listsTasks() {
    return getScheduler().listsTasks()
        .stream()
        .filter(this::includeRepoMoveTask)
        .collect(Collectors.toList());
  }

  private boolean includeRepoMoveTask(TaskInfo taskInfo) {
    return (changeRepoBlobstoreTaskEnabled || !taskInfo.getTypeId().equals(REPO_MOVE_TYPE_ID));
  }

  @Override
  public TaskInfo scheduleTask(final TaskConfiguration config, final Schedule schedule) {
    checkNotNull(config);
    checkNotNull(schedule);

    TaskDescriptor descriptor = checkNotNull(taskFactory.findDescriptor(config.getTypeId()));

    if (descriptor.isSingletonTaskType()) {
      TaskInfo existing = getTaskByTypeId(config.getTypeId());
      if (existing != null && !Objects.equals(existing.getId(), config.getId())) {
        log.warn("Singleton task type {} is already scheduled, rejecting duplicate", config.getTypeId());
        throw new ValidationErrorsException("Task " + config.getTypeId() + " already exists, ignoring");
      }
    }

    descriptor.completeConfiguration(config);
    config.validate();

    Date now = new Date();
    if (config.getCreated() == null) {
      config.setCreated(now);
    }
    config.setUpdated(now);

    TaskInfo taskInfo = getScheduler().scheduleTask(config, schedule);

    log.info("Task {} scheduled: {}",
        taskInfo.getConfiguration().getTaskLogName(),
        taskInfo.getSchedule().getType());

    eventManager.post(new TaskScheduledEvent(taskInfo));

    return taskInfo;
  }

  @Override
  public ExternalTaskState toExternalTaskState(final TaskInfo taskInfo) {
    return new ExternalTaskState(taskInfo);
  }

  @Override
  public boolean cancel(final String id, final boolean mayInterruptIfRunning) {
    return getScheduler().cancel(id, mayInterruptIfRunning);
  }

  @Nullable
  @Override
  public TaskInfo getTaskByTypeId(final String typeId) {
    return getScheduler().getTaskByTypeId(typeId);
  }

  @Nullable
  @Override
  public TaskInfo getTaskByTypeId(final String typeId, final Map<String, String> config) {
    return getScheduler().getTaskByTypeId(typeId, config);
  }

  @Override
  public boolean findAndSubmit(final String typeId) {
    return getScheduler().findAndSubmit(typeId);
  }

  @Override
  public boolean findWaitingTask(final String typeId, Map<String, String> config) {
    return getScheduler().findWaitingTask(typeId, config);
  }

  @Override
  public boolean findAndSubmit(final String typeId, final Map<String, String> config) {
    return getScheduler().findAndSubmit(typeId, config);
  }
}
