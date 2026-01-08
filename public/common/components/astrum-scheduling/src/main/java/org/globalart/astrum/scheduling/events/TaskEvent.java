
package org.globalart.astrum.scheduling.events;

import java.util.Date;

import javax.annotation.Nullable;

import org.globalart.astrum.scheduling.TaskInfo;
import org.globalart.astrum.scheduling.CurrentState;
import org.globalart.astrum.scheduling.LastRunState;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Abstract super class for task related events.
 *
 * @since 2.0
 */
public abstract class TaskEvent
{
  private final Date eventDate;

  private final TaskInfo taskInfo;

  private final CurrentState currentState;

  private final LastRunState lastRunState;

  public TaskEvent(final TaskInfo taskInfo) {
    this.eventDate = new Date();
    this.taskInfo = checkNotNull(taskInfo);
    this.currentState = taskInfo.getCurrentState();
    this.lastRunState = taskInfo.getLastRunState();
  }

  /**
   * Returns the timestamp of event creation.
   */
  public Date getEventDate() {
    return eventDate;
  }

  /**
   * Returns the "handle" of the task. Please note that this is "live" object, so states returned by this
   * instance may change as task progresses or even finishes it's work while the event handler gets this event.
   */
  public TaskInfo getTaskInfo() {
    return taskInfo;
  }

  /**
   * Gets the "current state" of the task in the moment this event was fired, never returns {@code null}.
   */
  public CurrentState getCurrentState() {
    return currentState;
  }

  /**
   * Gets the "last run state" in the moment this event was fired, may return {@code null}.
   */
  @Nullable
  public LastRunState getLastRunState() {
    return lastRunState;
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "astrumTask=" + getTaskInfo().getConfiguration() +
        ", currentState=" + currentState +
        ", lastRunState=" + lastRunState +
        '}';
  }
}
