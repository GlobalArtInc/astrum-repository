
package org.globalart.astrum.scheduling;

import java.util.Map;

import javax.annotation.Nullable;

import org.globalart.astrum.scheduling.schedule.Now;
import org.globalart.astrum.scheduling.schedule.Schedule;

import static java.util.Collections.emptyMap;

/**
 * The class holding information about task at the moment the instance of task info was created.
 *
 * This is the "handle" of a scheduled task.
 *
 * The handle might become "stale" if the task this instance is handle of is removed from scheduler
 * (ie. by some other thread). In that case, some of the methods will throw {@link TaskRemovedException} on invocation
 * to signal that state.
 *
 * For task entering {@link TaskState.Group#DONE}, this class will behave a bit differently:
 * they will never throw {@link TaskRemovedException}, and upon they are done, the task info will cache
 * task configuration, schedule, current and last run state forever.
 *
 * @since 3.0
 */
public interface TaskInfo
{
  /**
   * Returns a unique ID of the task instance.
   *
   * Shorthand method for {@link #getConfiguration()#getId()}
   */
  String getId();

  /**
   * Returns a name of the task instance.
   *
   * Shorthand method for {@link #getConfiguration()#getName()}
   */
  String getName();

  /**
   * Returns a type id of the task instance.
   *
   * Shorthand method for {@link #getConfiguration()#getTypeId()}
   *
   * @since 3.8
   */
  String getTypeId();

  /**
   * Returns a message of the task instance.
   *
   * Shorthand method for {@link #getConfiguration()#getMessage()}
   */
  String getMessage();

  /**
   * Returns a COPY of the task configuration map from the moment this instance was created or any state change on task
   * happened.
   *
   * Modifications to this configuration are possible, but does not affect currently executing task, nor is being
   * persisted.
   *
   * Generally, this configuration is only for inspection, or, to be used to re-schedule existing task with changed
   * configuration.
   */
  TaskConfiguration getConfiguration();

  /**
   * Returns the task's schedule from the moment this instance was created or any state change on task happened.
   */
  Schedule getSchedule();

  /**
   * Returns the task current state, never {@code null}.
   *
   * For tasks scheduled with {@link Now} schedule, or having manually started with {@link #runNow()} method,
   * the invocation of this method will block until underlying scheduler actually starts the task, hence,
   * caller might get the task result.
   */
  CurrentState getCurrentState();

  /**
   * Returns the task last run state, if there was any, otherwise {@code null}.
   */
  @Nullable
  default LastRunState getLastRunState() {
    return getConfiguration().getLastRunState();
  }

  /**
   * Returns the result object last returned by the task.
   */
  @Nullable
  Object getLastResult();

  /**
   * Removes (with canceling if runs) the task.
   *
   * Returns {@code true} if it's guaranteed that the task is removed from scheduler, and no future executions of
   * this task will happen. Still, task might be executing in this very moment, until detects cancellation.
   *
   * Returns {@code false} if task executes and is not cancelable.
   */
  boolean remove();

  /**
   * Executes the scheduled task now, unrelated to it's actual schedule. Already running task cannot have this method
   * executed, will throw {@link IllegalStateException}.
   *
   * This also implies that this method will NOT change the state of tasks "original" schedule!
   *
   * For example: hourly schedule, has 30 minutes to run, and task is getting run now. When the task is done,
   * there will be still 30 minutes for it's original schedule to run (minus the task execution time).
   *
   * @param triggerSource the source that triggered this task
   *
   * @throws TaskRemovedException if task with this ID has been removed from scheduler.
   * @throws IllegalStateException if task is already running
   *
   * @since 3.1
   */
  TaskInfo runNow(@Nullable String triggerSource) throws TaskRemovedException;

  /**
   * @see #runNow(String)
   */
  default TaskInfo runNow() throws TaskRemovedException {
    return runNow(null);
  }

  /**
   * Returns the source that triggered the current task execution, if known.
   *
   * @since 3.1
   */
  @Nullable
  String getTriggerSource();

  /**
   * Currently this context is passed through as-is to analytics, keep this in mind when adding data to the map
   *
   * @since 3.24
   */
  default Map<String, Object> getContext() {
    return emptyMap();
  }
}
