
package org.globalart.astrum.blobstore.metrics.reconcile;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.formfields.ComboboxFormField;
import org.globalart.astrum.scheduling.TaskDescriptorSupport;

import org.springframework.beans.factory.annotation.Value;

import static org.globalart.astrum.blobstore.common.BlobStoreTaskSupport.BLOBSTORE_NAME_FIELD_ID;
import static org.globalart.astrum.common.app.FeatureFlags.RECALCULATE_BLOBSTORE_SIZE_TASK_ENABLED_NAMED_VALUE;
import static org.globalart.astrum.formfields.FormField.MANDATORY;
import org.springframework.stereotype.Component;

/**
 * Task descriptor for {@link RecalculateBlobStoreSizeTask}
 */
@AvailabilityVersion(from = "1.0")
@Component
public class RecalculateBlobStoreSizeTaskDescriptor
    extends TaskDescriptorSupport
{
  public static final String TYPE_ID = "blobstore.metrics.reconcile";

  @Autowired
  public RecalculateBlobStoreSizeTaskDescriptor(
      @Value(RECALCULATE_BLOBSTORE_SIZE_TASK_ENABLED_NAMED_VALUE) final boolean taskEnabled)
  {
    super(TYPE_ID,
        RecalculateBlobStoreSizeTask.class,
        "Repair - Recalculate blob store storage",
        VISIBLE,
        taskEnabled,
        new ComboboxFormField<String>(
            BLOBSTORE_NAME_FIELD_ID,
            "Blob store",
            "Select the blob store(s) to recalculate",
            MANDATORY).withStoreApi("coreui_Blobstore.readNoneGroupEntriesIncludingEntryForAll")
                .withIdMapping("name"));
  }
}
