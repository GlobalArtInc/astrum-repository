
package org.globalart.astrum.logging.task;

import org.slf4j.Logger;
import org.slf4j.MDC;

/**
 * {@link TaskLogger} for logging to the task log, but also doing progress to the astrum.log.
 *
 * @since 3.6
 */
public class TaskLogWithProgressLogger
    extends TaskLogOnlyTaskLogger
    implements TaskLogger
{
  public TaskLogWithProgressLogger(final Logger log, final TaskLogInfo taskLogInfo) {
    super(log, taskLogInfo);
    MDC.put(TASK_LOG_WITH_PROGRESS_MDC, "true");
  }
}
