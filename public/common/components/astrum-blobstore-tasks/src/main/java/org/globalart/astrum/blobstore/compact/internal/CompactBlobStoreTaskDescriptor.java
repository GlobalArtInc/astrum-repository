
package org.globalart.astrum.blobstore.compact.internal;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.formfields.ComboboxFormField;
import org.globalart.astrum.formfields.NumberTextFormField;
import org.globalart.astrum.scheduling.TaskDescriptorSupport;

import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.formfields.FormField.MANDATORY;

/**
 * Task descriptor for {@link CompactBlobStoreTask}.
 *
 * @since 3.0
 */
@AvailabilityVersion(from = "1.0")
@Component
public class CompactBlobStoreTaskDescriptor
    extends TaskDescriptorSupport
{
  public static final String TYPE_ID = "blobstore.compact";

  public static final String BLOB_STORE_NAME_FIELD_ID = "blobstoreName";

  public static final String BLOBS_OLDER_THAN_FIELD_ID = "blobsOlderThan";

  @Autowired
  public CompactBlobStoreTaskDescriptor(
      @Value("${astrum.tasks.compactBlobstore.enabled:true}") final boolean visible)
  {
    super(TYPE_ID,
        CompactBlobStoreTask.class,
        "Admin - Compact blob store",
        visible,
        visible,
        new ComboboxFormField<String>(
            BLOB_STORE_NAME_FIELD_ID,
            "Blob store",
            "Select the blob store to compact",
            MANDATORY).withStoreApi("coreui_Blobstore.readWithAll").withIdMapping("name"),
        new NumberTextFormField(BLOBS_OLDER_THAN_FIELD_ID, "Blobs Older Than",
            "The number of days a blob should kept before permanent deletion (default 0)", false).withMinimumValue(0));
  }
}
