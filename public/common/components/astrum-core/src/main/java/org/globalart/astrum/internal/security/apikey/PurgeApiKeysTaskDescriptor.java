
package org.globalart.astrum.internal.security.apikey;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.scheduling.TaskDescriptorSupport;
import org.springframework.stereotype.Component;

/**
 * Task descriptor for {@link PurgeApiKeysTask}.
 *
 * @since 3.0
 */
@AvailabilityVersion(from = "1.0")
@Component
public class PurgeApiKeysTaskDescriptor
    extends TaskDescriptorSupport
{
  public static final String TYPE_ID = "security.purge-api-keys";

  public PurgeApiKeysTaskDescriptor() {
    super(TYPE_ID,
        PurgeApiKeysTask.class,
        "Admin - Delete orphaned API keys",
        VISIBLE,
        EXPOSED);
  }
}
