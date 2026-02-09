
package org.globalart.astrum.repository.content.tasks;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.formfields.FormField;
import org.globalart.astrum.formfields.RepositoryCombobox;
import org.globalart.astrum.repository.types.GroupType;
import org.globalart.astrum.repository.types.ProxyType;
import org.globalart.astrum.scheduling.TaskDescriptorSupport;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.RepositoryTaskSupport.REPOSITORY_NAME_FIELD_ID;

@AvailabilityVersion(from = "1.0")
@Component
public class BlobRepositoryMismatchTaskDescriptor
    extends TaskDescriptorSupport
{
  public static final String TYPE_ID = "repository.blob.mismatch.task";

  @Autowired
  public BlobRepositoryMismatchTaskDescriptor(
      @Value("${repository.blob.mismatch.task:false}") final boolean visibleExposed)
  {
    super(TYPE_ID, BlobRepositoryMismatchTask.class, "Find any blob with a mismatch on Bucket.repo-name and " +
        "fix it in the properties file",
        true, visibleExposed, visibleExposed,
        new RepositoryCombobox(
            REPOSITORY_NAME_FIELD_ID,
            "Repository",
            "Select the repository to fix blob mismatches",
            FormField.MANDATORY).includeAnEntryForAllRepositories()
                .excludingAnyOfTypes(GroupType.NAME)
                .excludingAnyOfTypes(ProxyType.NAME));
  }
}
