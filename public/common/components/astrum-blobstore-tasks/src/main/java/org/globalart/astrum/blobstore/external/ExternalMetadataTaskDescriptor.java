
package org.globalart.astrum.blobstore.external;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.formfields.RepositoryCombobox;
import org.globalart.astrum.formfields.StringTextFormField;
import org.globalart.astrum.repository.RepositoryTaskSupport;
import org.globalart.astrum.scheduling.TaskDescriptorSupport;

import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

/**
 * This task should only be run after an upgrade to 3.79 is finalized as its intended to be invoked in the context of an
 * upgrade step in the same context as the database upgrade we use 1.0 for the availability version.
 */
@AvailabilityVersion(from = "1.0")
@Component
public class ExternalMetadataTaskDescriptor
    extends TaskDescriptorSupport
{
  public static final String TYPE_ID = "external.blobstore.metadata";

  private static final String NAME = "Retrieve external blobstore metadata";

  @Autowired
  public ExternalMetadataTaskDescriptor(
      @Value("${external.blobstore.metadata:false}") final boolean visibleExposed)
  {
    super(TYPE_ID,
        ExternalMetadataTask.class,
        NAME,
        visibleExposed,
        visibleExposed,
        REQUEST_RECOVERY,
        new RepositoryCombobox(
            RepositoryTaskSupport.REPOSITORY_NAME_FIELD_ID,
            "Repository",
            "Select the repository to retrieve metadata for",
            true),
        new StringTextFormField(ExternalMetadataTask.FORMAT_FIELD_ID,
            "Repository Format",
            "Specify the repository format to retrieve metadata for",
            false));
  }
}
