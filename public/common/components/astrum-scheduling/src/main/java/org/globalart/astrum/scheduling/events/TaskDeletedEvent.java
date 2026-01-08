
package org.globalart.astrum.scheduling.events;

import org.globalart.astrum.scheduling.TaskInfo;

/**
 * Emitted when a task has been deleted.
 *
 * @since 3.1
 */
public class TaskDeletedEvent
    extends TaskEvent
{
  public TaskDeletedEvent(final TaskInfo taskInfo) {
    super(taskInfo);
  }
}
