package org.globalart.astrum.repository.yum.internal.task;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.formfields.RepositoryCombobox;
import org.globalart.astrum.repository.RepositoryTaskSupport;
import org.globalart.astrum.repository.types.HostedType;
import org.globalart.astrum.repository.yum.YumFormat;
import org.globalart.astrum.scheduling.TaskDescriptorSupport;

import org.springframework.stereotype.Component;

@AvailabilityVersion(from = "1.0")
@Component
public class AutomatedYumMetadataRebuildTaskDescriptor
    extends TaskDescriptorSupport
{
  public static final String TYPE_ID = "repository.yum.rebuild-metadata.automatic";

  public AutomatedYumMetadataRebuildTaskDescriptor() {
    super(TYPE_ID,
        AutomatedYumMetadataRebuildTask.class,
        "Automated Yum repository metadata rebuild",
        NOT_VISIBLE,
        NOT_EXPOSED,
        REQUEST_RECOVERY,
        new RepositoryCombobox(
            RepositoryTaskSupport.REPOSITORY_NAME_FIELD_ID,
            "Repository",
            "Select the Yum repository to rebuild metadata for",
            true).includingAnyOfFormats(YumFormat.NAME).includingAnyOfTypes(HostedType.NAME));
  }
}
