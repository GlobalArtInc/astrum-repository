
package org.globalart.astrum.scheduling.events;

import org.globalart.astrum.scheduling.TaskInfo;

/**
 * Event fired when a task failed with some error.
 *
 * @since 2.0
 */
public class TaskEventStoppedFailed
    extends TaskEventStopped
{
  /**
   * Failure cause.
   */
  private final Throwable throwable;

  public TaskEventStoppedFailed(
      final TaskInfo taskInfo,
      final Throwable throwable)
  {
    super(taskInfo);
    this.throwable = throwable;
  }

  /**
   * Returns the failure cause.
   */
  public Throwable getFailureCause() {
    return throwable;
  }
}
