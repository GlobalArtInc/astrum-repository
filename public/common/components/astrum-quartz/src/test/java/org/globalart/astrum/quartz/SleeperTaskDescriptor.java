
package org.globalart.astrum.quartz;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.scheduling.TaskDescriptorSupport;
import org.springframework.stereotype.Component;

/**
 * Descriptor for {@link SleeperTask}.
 */
@AvailabilityVersion(from = "1.0")
@Component
public class SleeperTaskDescriptor
    extends TaskDescriptorSupport
{
  public static final String TYPE_ID = "sleeper";

  public SleeperTaskDescriptor() {
    super(TYPE_ID, SleeperTask.class, "Sleeper test", VISIBLE, EXPOSED);
  }
}
