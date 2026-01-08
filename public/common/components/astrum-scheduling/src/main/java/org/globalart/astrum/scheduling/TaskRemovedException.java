
package org.globalart.astrum.scheduling;

import javax.annotation.Nullable;

/**
 * Checked exception thrown in cases when task is removed by some other party, but the caller is unaware of it.
 *
 * @since 3.0
 */
public class TaskRemovedException
    extends Exception
{
  public TaskRemovedException(final String taskId) {
    this(taskId, null);
  }

  public TaskRemovedException(final String taskId, @Nullable final Throwable cause) {
    super(String.format("Task '%s' does not exists", taskId), cause);
  }
}
