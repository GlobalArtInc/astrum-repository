
package org.globalart.astrum.scheduling.events;

import org.globalart.astrum.scheduling.TaskInfo;

/**
 * Event fired when a task is stopped, is cleanly done what was it doing.
 *
 * @since 2.0
 */
public class TaskEventStoppedDone
    extends TaskEventStopped
{
  public TaskEventStoppedDone(final TaskInfo taskInfo) {
    super(taskInfo);
  }
}
