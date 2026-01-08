
package org.globalart.astrum.scheduling.events;

import org.globalart.astrum.scheduling.TaskInfo;

/**
 * Event fired when a task is started (might be running or sleeping if blocked).
 *
 * @since 2.0
 */
public class TaskEventStarted
    extends TaskEvent
{
  public TaskEventStarted(final TaskInfo taskInfo) {
    super(taskInfo);
  }
}
