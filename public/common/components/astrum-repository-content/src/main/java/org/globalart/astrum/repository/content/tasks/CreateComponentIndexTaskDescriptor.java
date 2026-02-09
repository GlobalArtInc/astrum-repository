
package org.globalart.astrum.repository.content.tasks;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.scheduling.TaskDescriptorSupport;

import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

/**
 * This availability version is only set to 1.0 because the task is launched during upgrade process, before the db
 * migration version has been updated
 */
@AvailabilityVersion(from = "1.0")
@Component
public class CreateComponentIndexTaskDescriptor
    extends TaskDescriptorSupport
{
  public static final String NAME = "Repair - Recreate component indexes for performance";

  public static final String TYPE_ID = "create.component.index.task";

  public static final String EXPOSED_FLAG_VALUE = "${astrum.component.index.task.exposed:false}";

  public static final String VISIBLE_FLAG_VALUE = "${astrum.component.index.task.visible:false}";

  @Autowired
  public CreateComponentIndexTaskDescriptor(
      @Value(EXPOSED_FLAG_VALUE) final boolean exposed,
      @Value(VISIBLE_FLAG_VALUE) final boolean visible)
  {
    super(TYPE_ID, CreateComponentIndexTask.class, NAME, visible, exposed);
  }
}
