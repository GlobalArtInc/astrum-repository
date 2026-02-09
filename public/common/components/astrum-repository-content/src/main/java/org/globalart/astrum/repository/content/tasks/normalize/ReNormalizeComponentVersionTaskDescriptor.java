
package org.globalart.astrum.repository.content.tasks.normalize;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.scheduling.TaskDescriptorSupport;

import org.springframework.stereotype.Component;

/**
 * Descriptor for {@link ReNormalizeComponentVersionTask}. Purely an internal repair task submitted by
 * {@link org.globalart.astrum.repository.content.internal.ReNormalizationTaskLauncher} - never visible or exposed
 * to users, unlike {@link NormalizeComponentVersionTaskDescriptor}.
 */
@AvailabilityVersion(from = "1.0")
@Component
public class ReNormalizeComponentVersionTaskDescriptor
    extends TaskDescriptorSupport
{
  public static final String TYPE_ID = "component.renormalize.version";

  public ReNormalizeComponentVersionTaskDescriptor() {
    super(TYPE_ID,
        ReNormalizeComponentVersionTask.class,
        "Repair - Fix mismatched normalized component versions after normalizer change",
        false,
        false);
  }
}
