
package org.globalart.astrum.logging.task;

import org.slf4j.Marker;
import org.slf4j.MarkerFactory;

/**
 * Container for Marker constants
 *
 * @since 3.5
 */
public class TaskLoggingMarkers
{
  /**
   * Slf4j {@link Marker} to indicate the log should ONLY be logged to the astrum.log
   */
  public static final Marker ASTRUM_LOG_ONLY = MarkerFactory.getMarker("ASTRUM_LOG_ONLY");

  /**
   * Slf4j {@link Marker} to indicate the log should ONLY be logged to the task log
   */
  public static final Marker TASK_LOG_ONLY = MarkerFactory.getMarker("TASK_LOG_ONLY");

  /**
   * Slf4j {@link Marker} to indicate the log should ONLY be logged to the audit log
   */
  public static final Marker AUDIT_LOG_ONLY = MarkerFactory.getMarker("AUDIT_LOG_ONLY");

  /**
   * Slf4j {@link Marker} to indicate the log should ONLY be logged to the outbound requests log
   */
  public static final Marker OUTBOUND_REQUESTS_LOG_ONLY = MarkerFactory.getMarker("OUTBOUND_REQUESTS_LOG_ONLY");

  /**
   * Slf4j {@link Marker} to indicate the log should be logged with the progress logic (astrum.log only gets an entry
   * every minute, every entry goes to task log)
   */
  public static final Marker PROGRESS = MarkerFactory.getMarker("PROGRESS");

  /**
   * FOR INTERNAL USE ONLY - DO NOT USE IN A TASK
   * Slf4j {@link Marker} used internally by the task logging progress mechanism.
   */
  public static final Marker INTERNAL_PROGRESS = MarkerFactory.getMarker("INTERNAL_PROGRESS");
}
