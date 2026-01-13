
package org.globalart.astrum.audit.internal.store;

import java.util.Comparator;
import java.util.Date;
import java.util.List;
import java.util.function.Predicate;

import org.globalart.astrum.common.app.ManagedLifecycle;
import org.globalart.astrum.common.stateguard.StateGuardLifecycleSupport;
import org.globalart.astrum.scheduling.TaskConfiguration;
import org.globalart.astrum.scheduling.TaskInfo;
import org.globalart.astrum.scheduling.TaskScheduler;
import org.globalart.astrum.scheduling.schedule.Schedule;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.app.FeatureFlags.AUDIT_EVENTS_CLEANUP_TASK_ENABLED;
import static org.globalart.astrum.common.app.FeatureFlags.AUDIT_EVENTS_CLEANUP_TASK_ENABLED_NAMED_VALUE;
import static org.globalart.astrum.common.app.ManagedLifecycle.Phase.TASKS;

/**
 * Auto-creates the {@link AuditEventCleanupTask} daily schedule at startup unless a task of the
 * same type already exists. Any custom schedule an operator has configured via the tasks UI is
 * preserved across restarts.
 */
@Component
@ManagedLifecycle(phase = TASKS)
public class AuditEventCleanupTaskManager
    extends StateGuardLifecycleSupport
{
  private final TaskScheduler taskScheduler;

  private final String cleanupCron;

  private final boolean enabled;

  private final Predicate<TaskInfo> isCleanupTask =
      info -> AuditEventCleanupTaskDescriptor.TYPE_ID.equals(info.getConfiguration().getTypeId());

  @Autowired
  public AuditEventCleanupTaskManager(
      final TaskScheduler taskScheduler,
      @Value("${astrum.audit.events.cleanup.cron:0 30 3 * * ?}") final String cleanupCron,
      @Value(AUDIT_EVENTS_CLEANUP_TASK_ENABLED_NAMED_VALUE) final boolean enabled)
  {
    this.taskScheduler = checkNotNull(taskScheduler);
    this.cleanupCron = checkNotNull(cleanupCron);
    this.enabled = enabled;
  }

  @Override
  protected void doStart() throws Exception {
    removeDuplicateTasks();

    if (!enabled) {
      log.info("Audit_events cleanup task disabled by feature flag ({}=false); skipping auto-registration",
          AUDIT_EVENTS_CLEANUP_TASK_ENABLED);
      return;
    }

    TaskInfo taskInfo = taskScheduler.getTaskByTypeId(AuditEventCleanupTaskDescriptor.TYPE_ID);
    if (taskInfo == null) {
      TaskConfiguration configuration =
          taskScheduler.createTaskConfigurationInstance(AuditEventCleanupTaskDescriptor.TYPE_ID);
      configuration.setName(AuditEventCleanupTaskDescriptor.TASK_NAME);
      Schedule schedule = taskScheduler.getScheduleFactory().cron(new Date(), this.cleanupCron);
      taskScheduler.scheduleTask(configuration, schedule);
      log.info("Created audit_events cleanup task with default schedule: {}", this.cleanupCron);
    }
    else {
      log.debug("Audit cleanup task already exists, preserving existing schedule");
    }
  }

  private void removeDuplicateTasks() {
    List<TaskInfo> tasks = taskScheduler.listsTasks()
        .stream()
        .filter(isCleanupTask)
        .sorted(Comparator.comparing(TaskInfo::getId))
        .skip(1)
        .toList();

    if (!tasks.isEmpty()) {
      log.warn("Found {} duplicate audit cleanup tasks, removing extras", tasks.size());
      tasks.forEach(TaskInfo::remove);
    }
  }
}
