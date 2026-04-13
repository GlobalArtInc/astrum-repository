
package org.globalart.astrum.repository.apt.internal.upgrade;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.scheduling.TaskDescriptorSupport;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

@AvailabilityVersion(from = "1.0")
@Component
public class RepairAptMetadataLeadingSlashTaskDescriptor
    extends TaskDescriptorSupport
{
  private static final String TASK_NAME = "Upgrade - Fix apt filenames with leading slash";

  static final String TASK_DESCRIPTION = "Updates assets stored with Filename using a leading slash";

  static final String TYPE_ID = "apt.upgrade.filename.leading.slash";

  @Autowired
  public RepairAptMetadataLeadingSlashTaskDescriptor(
      @Value("${apt.upgrade.filename.leading.slash:false}") final boolean visible)
  {
    super(TYPE_ID, RepairAptMetadataLeadingSlashTask.class, TASK_NAME, visible, visible);
  }
}
