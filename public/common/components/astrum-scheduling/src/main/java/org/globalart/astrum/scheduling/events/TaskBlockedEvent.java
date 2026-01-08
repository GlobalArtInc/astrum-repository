
package org.globalart.astrum.scheduling.events;

import org.globalart.astrum.scheduling.TaskInfo;

/**
 * Emitted when a task has been blocked waiting for other tasks to finish.
 *
 * @since 3.1
 */
public class TaskBlockedEvent
    extends TaskEvent
{
  public TaskBlockedEvent(final TaskInfo taskInfo) {
    super(taskInfo);
  }
}
