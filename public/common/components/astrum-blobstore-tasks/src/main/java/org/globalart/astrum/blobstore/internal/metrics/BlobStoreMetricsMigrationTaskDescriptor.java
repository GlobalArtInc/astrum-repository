
package org.globalart.astrum.blobstore.internal.metrics;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.formfields.ComboboxFormField;
import org.globalart.astrum.scheduling.TaskDescriptorSupport;

import org.springframework.beans.factory.annotation.Value;

import static org.globalart.astrum.blobstore.common.BlobStoreTaskSupport.BLOBSTORE_NAME_FIELD_ID;
import static org.globalart.astrum.formfields.FormField.MANDATORY;
import org.springframework.stereotype.Component;

@AvailabilityVersion(from = "2.0")
@Component
public class BlobStoreMetricsMigrationTaskDescriptor
    extends TaskDescriptorSupport
{
  private static final String EXPOSED_FLAG_VALUE = "${astrum.blobstore.metrics.migration.task.expose:false}";

  @Autowired
  public BlobStoreMetricsMigrationTaskDescriptor(
      @Value(EXPOSED_FLAG_VALUE) final boolean exposed)
  {
    super(BlobStoreMetricsMigrationTask.TYPE_ID,
        BlobStoreMetricsMigrationTask.class,
        "Migration - Move blobstore metrics to the database",
        VISIBLE,
        exposed,
        new ComboboxFormField<String>(
            BLOBSTORE_NAME_FIELD_ID,
            "Blob store",
            "Select the blob store(s) to obtain metrics for",
            MANDATORY)
                .withStoreApi("coreui_Blobstore.readNoneGroupEntriesIncludingEntryForAll")
                .withIdMapping("name"));
  }
}
