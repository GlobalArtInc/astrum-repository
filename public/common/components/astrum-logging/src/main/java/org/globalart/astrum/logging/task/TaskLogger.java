
package org.globalart.astrum.logging.task;

/**
 * @since 3.5
 */
public interface TaskLogger
{
  // id used in discriminator. See logback.xml
  String LOGBACK_TASK_DISCRIMINATOR_ID = "taskIdAndDate";

  // constant for MDC use
  String TASK_LOG_ONLY_MDC = "TASK_LOG_ONLY_MDC";

  // constant for MDC use
  String TASK_LOG_WITH_PROGRESS_MDC = "TASK_LOG_WITH_PROGRESS_MDC";

  /**
   * Required to start the task logging. See {@link TaskLoggerHelper#start(TaskLogger)}
   */
  void start();

  /**
   * Required to close out the task logging. This involves cleaning up MDC and ThreadLocal variables. See {@link
   * TaskLoggerHelper#finish()}
   */
  void finish();

  /**
   * Log a progress event, which are always logged to the task log, but only periodically to the astrum.log
   *
   * @param event log event containing progress
   */
  void progress(TaskLoggingEvent event);

  /**
   * Flush any pending progress messages so they are logged immediately
   */
  void flush();
}
