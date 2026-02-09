
package org.globalart.astrum.repository.content.tasks.normalize;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.scheduling.TaskDescriptorSupport;

import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

/**
 * Descriptor for {@link NormalizeComponentVersionTask} which populates the normalized_version column on the
 * {format}_component tables
 */
@AvailabilityVersion(from = "1.0")
@Component
public class NormalizeComponentVersionTaskDescriptor
    extends TaskDescriptorSupport
{
  public static final String TYPE_ID = "component.normalize.version";

  public static final String EXPOSED_FLAG_VALUE = "${astrum.component.normalize.expose:false}";

  public static final String VISIBLE_FLAG_VALUE = "${astrum.component.normalize.visible:false}";

  @Autowired
  public NormalizeComponentVersionTaskDescriptor(
      @Value(EXPOSED_FLAG_VALUE) final boolean exposed,
      @Value(VISIBLE_FLAG_VALUE) final boolean visible)
  {
    super(TYPE_ID,
        NormalizeComponentVersionTask.class,
        "Repair - Normalize component versions for retain-n",
        visible,
        exposed);
  }
}
