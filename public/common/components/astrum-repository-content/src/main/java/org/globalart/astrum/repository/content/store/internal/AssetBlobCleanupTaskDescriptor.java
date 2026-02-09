
package org.globalart.astrum.repository.content.store.internal;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.scheduling.TaskDescriptorSupport;
import org.springframework.stereotype.Component;

/**
 * @since 3.24
 */
@AvailabilityVersion(from = "1.0")
@Component
public class AssetBlobCleanupTaskDescriptor
    extends TaskDescriptorSupport
{
  public static final String TYPE_ID = "assetBlob.cleanup";

  public static final String FORMAT_FIELD_ID = "format";

  public static final String CONTENT_STORE_FIELD_ID = "contentStore";

  public AssetBlobCleanupTaskDescriptor() {
    super(TYPE_ID,
        AssetBlobCleanupTask.class,
        "Admin - Cleanup unused asset blobs",
        VISIBLE,
        NOT_EXPOSED);
  }
}
