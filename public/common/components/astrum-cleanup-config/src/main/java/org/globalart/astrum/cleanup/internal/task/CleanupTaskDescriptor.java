
package org.globalart.astrum.cleanup.internal.task;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.scheduling.TaskDescriptorSupport;
import org.springframework.stereotype.Component;

/**
 * Task descriptor for {@link CleanupTask}.
 *
 * @since 3.14
 */
@AvailabilityVersion(from = "1.0")
@Component
public class CleanupTaskDescriptor
    extends TaskDescriptorSupport
{
  public static final String TYPE_ID = "repository.cleanup";

  public CleanupTaskDescriptor() {
    super(TYPE_ID,
        CleanupTask.class,
        "Admin - Cleanup repositories using their associated policies",
        VISIBLE,
        NOT_EXPOSED);
  }
}
