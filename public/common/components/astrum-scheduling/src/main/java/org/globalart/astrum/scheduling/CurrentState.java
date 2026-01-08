
package org.globalart.astrum.scheduling;

import java.util.Date;
import java.util.concurrent.Future;

import javax.annotation.Nullable;

/**
 * Encapsulates the current state of a task.
 * 
 * @since 3.19
 */
public interface CurrentState
{
  /**
   * Returns the state of task, never {@code null}.
   */
  TaskState getState();

  /**
   * Returns the date of next run, if applicable, or {@code null}.
   */
  @Nullable
  Date getNextRun();

  /**
   * If task is running, returns it's run state, otherwise {@code null}.
   */
  @Nullable
  Date getRunStarted();

  /**
   * If task is running, returns it's run state, otherwise {@code null}.
   */
  @Nullable
  TaskState getRunState();

  /**
   * If task is in states {@link TaskState.Group#RUNNING} or {@link TaskState.Group#DONE}, returns it's future,
   * otherwise {@code null}.
   * In case of {@link TaskState.Group#DONE} the future is done too.
   */
  @Nullable
  Future<?> getFuture();
}
