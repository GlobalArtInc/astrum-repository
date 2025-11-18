
package org.globalart.astrum.logging.task;

import org.slf4j.Logger;
import org.slf4j.MDC;

/**
 * Each task is executed in its own thread and its {@link TaskLogger} is stored in it here.
 *
 * @since 3.5
 */
public class TaskLoggerHelper
{
  private static final ThreadLocal<TaskLogger> context = new ThreadLocal<>();

  private TaskLoggerHelper() {
    throw new IllegalAccessError("Utility class");
  }

  public static void start(final TaskLogger taskLogger) {
    taskLogger.start();
    context.set(taskLogger);
  }

  public static TaskLogger get() {
    return context.get();
  }

  public static void finish() {
    TaskLogger taskLogger = get();
    if (taskLogger != null) {
      taskLogger.finish();
    }
    context.remove();
  }

  public static void progress(final TaskLoggingEvent event) {
    TaskLogger taskLogger = get();
    if (taskLogger != null) {
      taskLogger.progress(event);
    }
  }

  public static void progress(final Logger logger, final String message, Object... args) {
    progress(new TaskLoggingEvent(logger, message, args));
  }

  /**
   * @see TaskLogger#flush()
   */
  public static void flush() {
    TaskLogger taskLogger = get();
    if (taskLogger != null) {
      taskLogger.flush();
    }
  }

  /**
   * Joins to the task logger without the overhead of creating a new task logger. This is used when a task creates
   * a new thread and wants to log to the same task logger.
   * 
   * @param discriminator
   */
  public static void joinTaskLogger(String discriminator) {
    MDC.put(TaskLogger.LOGBACK_TASK_DISCRIMINATOR_ID, discriminator);
  }

  /**
   * Get the current task log discriminator.
   * 
   * @return
   */
  public static String getCurrentTaskDiscriminator() {
    return MDC.get(TaskLogger.LOGBACK_TASK_DISCRIMINATOR_ID);
  }
}
