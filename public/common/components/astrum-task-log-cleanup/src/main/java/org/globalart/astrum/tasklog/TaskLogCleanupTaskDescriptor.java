
package org.globalart.astrum.tasklog;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.scheduling.TaskDescriptorSupport;
import org.springframework.stereotype.Component;

/**
 * @since 3.5
 */
@AvailabilityVersion(from = "1.0")
@Component
public class TaskLogCleanupTaskDescriptor
    extends TaskDescriptorSupport
{
  public static final String TYPE_ID = "tasklog.cleanup";

  @Autowired
  public TaskLogCleanupTaskDescriptor() {
    super(TYPE_ID, TaskLogCleanupTask.class, "Task log cleanup", TaskDescriptorSupport.NOT_VISIBLE,
        TaskDescriptorSupport.NOT_EXPOSED);
  }
}
