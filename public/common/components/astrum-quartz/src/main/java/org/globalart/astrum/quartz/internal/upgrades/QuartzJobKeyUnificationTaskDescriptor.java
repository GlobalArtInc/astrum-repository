
package org.globalart.astrum.quartz.internal.upgrades;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.scheduling.TaskDescriptorSupport;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

@AvailabilityVersion(from = "1.0")
@Component
public class QuartzJobKeyUnificationTaskDescriptor
    extends TaskDescriptorSupport
{
  static final String TYPE_ID = "quartz.jobkey.unification";

  static final String VISIBLE_FLAG_VALUE = "${astrum.quartz.jobkey.unification.visible:false}";

  @Autowired
  public QuartzJobKeyUnificationTaskDescriptor(@Value(VISIBLE_FLAG_VALUE) boolean visible) {
    super(TYPE_ID,
        QuartzJobKeyUnificationTask.class,
        "Quartz - Job Key Unification",
        visible,
        visible);
  }
}
