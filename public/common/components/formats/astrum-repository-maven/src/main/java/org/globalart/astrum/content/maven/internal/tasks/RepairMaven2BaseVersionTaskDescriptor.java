
package org.globalart.astrum.content.maven.internal.tasks;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.formfields.FormField;
import org.globalart.astrum.formfields.RepositoryCombobox;
import org.globalart.astrum.repository.maven.internal.Maven2Format;
import org.globalart.astrum.repository.types.HostedType;
import org.globalart.astrum.scheduling.TaskDescriptorSupport;

import static org.globalart.astrum.repository.RepositoryTaskSupport.REPOSITORY_NAME_FIELD_ID;
import org.springframework.stereotype.Component;

@AvailabilityVersion(from = "1.0")
@Component
public class RepairMaven2BaseVersionTaskDescriptor
    extends TaskDescriptorSupport
{
  public static final String TASK_NAME = "Maven - Repair Base Version";

  public static final String TYPE_ID = "repository.maven.repair-base-version";

  public RepairMaven2BaseVersionTaskDescriptor() {
    super(TYPE_ID,
        RepairMaven2BaseVersionTask.class,
        TASK_NAME,
        VISIBLE,
        EXPOSED,
        new RepositoryCombobox(
            REPOSITORY_NAME_FIELD_ID,
            "Repository",
            "Select the repository to repair base version in",
            FormField.MANDATORY).includingAnyOfFormats(Maven2Format.NAME)
                .includingAnyOfTypes(HostedType.NAME)
                .includeAnEntryForAllRepositories());
  }
}
