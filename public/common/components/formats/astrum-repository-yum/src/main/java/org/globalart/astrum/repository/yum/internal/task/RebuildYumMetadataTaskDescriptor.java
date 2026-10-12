package org.globalart.astrum.repository.yum.internal.task;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.formfields.CheckboxFormField;
import org.globalart.astrum.formfields.RepositoryCombobox;
import org.globalart.astrum.repository.RepositoryTaskSupport;
import org.globalart.astrum.repository.types.HostedType;
import org.globalart.astrum.repository.yum.YumFormat;
import org.globalart.astrum.scheduling.TaskDescriptorSupport;

import org.springframework.stereotype.Component;

import static org.globalart.astrum.formfields.FormField.OPTIONAL;

@AvailabilityVersion(from = "1.0")
@Component
public class RebuildYumMetadataTaskDescriptor
    extends TaskDescriptorSupport
{
  public static final String TYPE_ID = "repository.yum.rebuild.metadata";

  public static final String SOFT_REPAIR_FIELD_ID = "yumMetadataCaching";

  public RebuildYumMetadataTaskDescriptor() {
    super(TYPE_ID,
        RebuildYumMetadataTask.class,
        "Repair - Rebuild Yum repository metadata (repodata)",
        VISIBLE,
        EXPOSED,
        new RepositoryCombobox(
            RepositoryTaskSupport.REPOSITORY_NAME_FIELD_ID,
            "Repository",
            "Select the Yum repository to rebuild metadata for",
            true).includingAnyOfFormats(YumFormat.NAME).includingAnyOfTypes(HostedType.NAME),
        new CheckboxFormField(
            SOFT_REPAIR_FIELD_ID,
            "Soft repair",
            "Only update the metadata with RPM changes since the last metadata generation",
            OPTIONAL));
  }
}
