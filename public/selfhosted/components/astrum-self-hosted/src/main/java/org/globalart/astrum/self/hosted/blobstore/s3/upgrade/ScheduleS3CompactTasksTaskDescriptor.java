
package org.globalart.astrum.self.hosted.blobstore.s3.upgrade;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.scheduling.TaskDescriptorSupport;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

@AvailabilityVersion(from = "1.0")
@Component
public class ScheduleS3CompactTasksTaskDescriptor
    extends TaskDescriptorSupport
{
  public static final String TYPE_ID = "s3.compact.task.scheduling.migration";

  @Autowired
  public ScheduleS3CompactTasksTaskDescriptor(
      @Value("${s3.compact.task.scheduling.exposed:false}") final boolean exposed)
  {
    super(TYPE_ID, ScheduleS3CompactTasksTask.class, "Schedule compact blobstore tasks based on expiration policy",
        exposed, exposed, true);
  }
}
