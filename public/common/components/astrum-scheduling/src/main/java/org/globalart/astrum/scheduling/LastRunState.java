
package org.globalart.astrum.scheduling;

import java.util.Date;

/**
 * Encapsulates the end state of the last run of a task.
 *
 * @since 3.19
 */
public interface LastRunState
{
  /**
   * Returns the last end state.
   */
  TaskState getEndState();

  /**
   * Returns the date when the task actually started running (after any blocking/waiting).
   */
  Date getRunStarted();

  /**
   * Returns the date when the task was scheduled to run (may be earlier than runStarted if task was blocked).
   */
  Date getRunScheduled();

  /**
   * Returns the last run duration.
   */
  long getRunDuration();
}
