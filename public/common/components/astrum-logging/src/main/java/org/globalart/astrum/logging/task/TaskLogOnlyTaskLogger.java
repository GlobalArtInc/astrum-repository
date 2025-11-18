
package org.globalart.astrum.logging.task;

import org.slf4j.Logger;
import org.slf4j.MDC;

/**
 * {@link TaskLogger} for logging just to the task log. Stores a value in {@link MDC} for AstrumLogFilter to find.
 *
 * @since 3.5
 */
public class TaskLogOnlyTaskLogger
    extends SeparateTaskLogTaskLogger
{
  TaskLogOnlyTaskLogger(final Logger log, final TaskLogInfo taskLogInfo) {
    super(log, taskLogInfo);
    MDC.put(TASK_LOG_ONLY_MDC, "true");
  }

  @Override
  protected void writeLogFileNameToAstrumLog() {
    MDC.remove(TASK_LOG_ONLY_MDC);
    super.writeLogFileNameToAstrumLog();
    MDC.put(TASK_LOG_ONLY_MDC, "true");
  }
}
