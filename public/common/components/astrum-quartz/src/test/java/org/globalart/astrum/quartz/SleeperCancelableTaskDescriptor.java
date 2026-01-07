
package org.globalart.astrum.quartz;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.scheduling.TaskDescriptorSupport;
import org.springframework.stereotype.Component;

/**
 * Descriptor for {@link SleeperCancelableTask}.
 */
@AvailabilityVersion(from = "1.0")
@Component
public class SleeperCancelableTaskDescriptor
    extends TaskDescriptorSupport
{
  static final String TYPE_ID = "sleeperCancelable";

  public SleeperCancelableTaskDescriptor() {
    super(TYPE_ID, SleeperCancelableTask.class, "Sleeper cancelable test", VISIBLE, EXPOSED);
  }
}
