
package org.globalart.astrum.tasklog;

import java.util.Date;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.app.ManagedLifecycle;
import org.globalart.astrum.common.stateguard.StateGuardLifecycleSupport;
import org.globalart.astrum.scheduling.TaskConfiguration;
import org.globalart.astrum.scheduling.TaskScheduler;
import org.globalart.astrum.scheduling.schedule.Schedule;

import org.springframework.beans.factory.annotation.Value;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.app.ManagedLifecycle.Phase.TASKS;
import org.springframework.stereotype.Component;

/**
 * Adds the {@link TaskLogCleanupTask} to the quartz cron definition in the database
 *
 * @since 3.5
 */
@Component
@ManagedLifecycle(phase = TASKS)
public class TaskLogCleanupQuartz
    extends StateGuardLifecycleSupport
{
  private final TaskScheduler taskScheduler;

  private final String taskLogCleanupCron;

  @Autowired
  public TaskLogCleanupQuartz(
      final TaskScheduler taskScheduler,
      @Value("${astrum.tasks.log.cleanup.cron:0 0 0 * * ?}") final String taskLogCleanupCron)
  {
    this.taskScheduler = checkNotNull(taskScheduler);
    this.taskLogCleanupCron = checkNotNull(taskLogCleanupCron);
  }

  @Override
  protected void doStart() throws Exception {
    if (!taskScheduler.listsTasks()
        .stream()
        .anyMatch((info) -> TaskLogCleanupTaskDescriptor.TYPE_ID.equals(info.getConfiguration().getTypeId()))) {
      TaskConfiguration configuration = taskScheduler.createTaskConfigurationInstance(
          TaskLogCleanupTaskDescriptor.TYPE_ID);
      Schedule schedule = taskScheduler.getScheduleFactory().cron(new Date(), taskLogCleanupCron);
      taskScheduler.scheduleTask(configuration, schedule);
    }
  }
}
