
package org.globalart.astrum.repository.content.store.internal.migration;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.scheduling.TaskDescriptorSupport;
import org.springframework.stereotype.Component;

/**
 * {@link AssetBlobRefMigrationTask} descriptor.
 */
@AvailabilityVersion(from = "1.0")
@Component
public class AssetBlobRefMigrationTaskDescriptor
    extends TaskDescriptorSupport
{
  public static final String TYPE_ID = "assetBlob.blobRef.migration";

  public static final String FORMAT_FIELD_ID = "format";

  public static final String CONTENT_STORE_FIELD_ID = "contentStore";

  public AssetBlobRefMigrationTaskDescriptor() {
    super(TYPE_ID,
        AssetBlobRefMigrationTask.class,
        "Admin - Migrate blobRef to new column",
        NOT_VISIBLE,
        NOT_EXPOSED,
        REQUEST_RECOVERY);
  }
}
