
package org.globalart.astrum.scheduling;

/**
 * Generates notification messages for tasks.
 *
 * @since 3.22
 */
public interface TaskNotificationMessageGenerator
{

  String completed(TaskInfo taskInfo);

  String failed(TaskInfo taskInfo, Throwable cause);
}
