
package org.globalart.astrum.cleanup.internal.task;

import java.util.Date;
import java.util.List;
import java.util.function.Predicate;
import java.util.stream.Collectors;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.lifecycle.LifecycleSupport;
import org.globalart.astrum.common.app.ManagedLifecycle;
import org.globalart.astrum.scheduling.TaskConfiguration;
import org.globalart.astrum.scheduling.TaskInfo;
import org.globalart.astrum.scheduling.TaskScheduler;
import org.globalart.astrum.scheduling.schedule.Cron;
import org.globalart.astrum.scheduling.schedule.Schedule;

import com.google.common.annotations.VisibleForTesting;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.app.ManagedLifecycle.Phase.TASKS;
import org.springframework.stereotype.Component;

/**
 * Instantiates the cleanup task on system startup if it does not already exist
 *
 * @since 3.14
 */
@Component
@ManagedLifecycle(phase = TASKS)
public class CleanupBootService
    extends LifecycleSupport
{
  @VisibleForTesting
  static final String CRON = "0 0 1 * * ?";

  @VisibleForTesting
  static final String TASK_NAME = "Cleanup service";

  private final TaskScheduler taskScheduler;

  @Autowired
  public CleanupBootService(final TaskScheduler taskScheduler) {
    this.taskScheduler = checkNotNull(taskScheduler);
  }

  @Override
  protected void doStart() {
    createCleanupTask();
  }

  private void createCleanupTask() {
    if (!doesTaskExist()) {
      TaskConfiguration taskConfig = taskScheduler.createTaskConfigurationInstance(CleanupTaskDescriptor.TYPE_ID);
      taskConfig.setName(TASK_NAME);
      try {
        Cron run1amEveryDay = taskScheduler.getScheduleFactory().cron(new Date(), CRON);
        taskScheduler.scheduleTask(taskConfig, run1amEveryDay);
      }
      catch (RuntimeException e) {
        log.error("Problem scheduling cleanup task", e);
      }
    }
    removeDuplicates();
  }

  private void removeDuplicates() {
    List<TaskInfo> tasks = taskScheduler.listsTasks()
        .stream()
        .filter(isCleanupTask())
        .filter(info -> TASK_NAME.equals(info.getConfiguration().getName()))
        .filter(scheduleMatches())
        .collect(Collectors.toList());

    if (tasks.size() > 1) {
      tasks.subList(1, tasks.size()).forEach(TaskInfo::remove);
    }
  }

  private boolean doesTaskExist() {
    return taskScheduler.listsTasks().stream().anyMatch(isCleanupTask());
  }

  private static Predicate<TaskInfo> isCleanupTask() {
    return info -> CleanupTaskDescriptor.TYPE_ID.equals(info.getConfiguration().getTypeId());
  }

  private Predicate<TaskInfo> scheduleMatches() {
    Cron cron = taskScheduler.getScheduleFactory().cron(new Date(), CRON);
    return taskInfo -> {
      Schedule schedule = taskInfo.getSchedule();
      return schedule instanceof Cron && cron.getCronExpression().equals(((Cron) schedule).getCronExpression());
    };
  }
}
