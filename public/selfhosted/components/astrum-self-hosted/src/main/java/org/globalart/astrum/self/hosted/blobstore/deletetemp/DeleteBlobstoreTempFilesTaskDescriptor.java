
package org.globalart.astrum.self.hosted.blobstore.deletetemp;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.formfields.ComboboxFormField;
import org.globalart.astrum.formfields.NumberTextFormField;
import org.globalart.astrum.scheduling.TaskDescriptorSupport;

import static org.globalart.astrum.formfields.FormField.MANDATORY;
import static org.globalart.astrum.formfields.FormField.OPTIONAL;
import org.springframework.stereotype.Component;

@AvailabilityVersion(from = "1.0")
@Component
public class DeleteBlobstoreTempFilesTaskDescriptor
    extends TaskDescriptorSupport
{
  public static final String TYPE_ID = "blobstore.delete-temp-files";

  public static final String BLOB_STORE_NAME_FIELD_ID = "blobstoreName";

  public static final String DAYS_OLDER_THAN = "olderThanDays";

  @Autowired
  public DeleteBlobstoreTempFilesTaskDescriptor() {
    super(TYPE_ID,
        DeleteBlobstoreTempFilesTask.class,
        "Admin - Delete blob store temporary files",
        VISIBLE,
        EXPOSED,
        new ComboboxFormField<String>(
            BLOB_STORE_NAME_FIELD_ID,
            "Blob store",
            "Select the blob store to delete temporary files",
            MANDATORY).withStoreApi("coreui_Blobstore.read").withIdMapping("name"),
        new NumberTextFormField(DAYS_OLDER_THAN,
            "Delete Files Older Than X days",
            "Delete files that are older than X days", OPTIONAL).withMinimumValue(0));
  }
}
