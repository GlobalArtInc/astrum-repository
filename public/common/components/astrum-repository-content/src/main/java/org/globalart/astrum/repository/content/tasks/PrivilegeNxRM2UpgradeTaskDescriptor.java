
package org.globalart.astrum.repository.content.tasks;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.scheduling.TaskDescriptorSupport;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

@AvailabilityVersion(from = "1.0")
@Component
public class PrivilegeNxRM2UpgradeTaskDescriptor
    extends TaskDescriptorSupport
{
  public static final String TYPE_ID = "nxrm2.privilege.upgrade";

  @Autowired
  public PrivilegeNxRM2UpgradeTaskDescriptor() {
    super(TYPE_ID, PrivilegeNxRM2UpgradeTask.class,
        "Upgrade NxRM2 privileges Task",
        TaskDescriptorSupport.NOT_VISIBLE,
        TaskDescriptorSupport.NOT_EXPOSED);
  }
}
