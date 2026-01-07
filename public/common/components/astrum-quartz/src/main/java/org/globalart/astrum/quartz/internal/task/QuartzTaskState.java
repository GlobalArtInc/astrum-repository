
package org.globalart.astrum.quartz.internal.task;

import java.util.Date;

import javax.annotation.Nullable;

import org.globalart.astrum.scheduling.LastRunState;
import org.globalart.astrum.scheduling.Task;
import org.globalart.astrum.scheduling.TaskConfiguration;
import org.globalart.astrum.scheduling.schedule.Schedule;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * State of Quartz executed {@link Task}.
 *
 * @since 3.0
 */
public class QuartzTaskState
{
  private final TaskConfiguration taskConfiguration;

  private final Schedule schedule;

  private final Date nextExecutionTime;

  public QuartzTaskState(
      final TaskConfiguration config,
      final Schedule schedule,
      @Nullable final Date nextExecutionTime)
  {
    this.taskConfiguration = checkNotNull(config);
    this.schedule = checkNotNull(schedule);
    this.nextExecutionTime = nextExecutionTime;
  }

  public TaskConfiguration getConfiguration() {
    return taskConfiguration;
  }

  public Schedule getSchedule() {
    return schedule;
  }

  public Date getNextExecutionTime() {
    return nextExecutionTime;
  }

  @Nullable
  public LastRunState getLastRunState() {
    return taskConfiguration.getLastRunState();
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "taskConfiguration=" + taskConfiguration +
        ", schedule=" + schedule +
        ", nextExecutionTime=" + nextExecutionTime +
        '}';
  }
}
