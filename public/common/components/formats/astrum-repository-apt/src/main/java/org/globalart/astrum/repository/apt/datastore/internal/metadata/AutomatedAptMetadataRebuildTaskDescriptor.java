
package org.globalart.astrum.repository.apt.datastore.internal.metadata;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.formfields.RepositoryCombobox;
import org.globalart.astrum.repository.RepositoryTaskSupport;
import org.globalart.astrum.repository.apt.AptFormat;
import org.globalart.astrum.repository.types.HostedType;
import org.globalart.astrum.repository.types.ProxyType;
import org.globalart.astrum.scheduling.TaskDescriptorSupport;

import org.springframework.stereotype.Component;

/**
 * Base class for automated APT metadata rebuild task descriptors.
 * Provides common configuration for both hosted and proxy repository metadata rebuild tasks.
 * <p>
 * These tasks are NOT_VISIBLE and NOT_EXPOSED - they are only used internally by the scheduler
 * to automatically rebuild metadata in response to repository events.
 */
@AvailabilityVersion(from = "1.0")
@Component
public class AutomatedAptMetadataRebuildTaskDescriptor
    extends TaskDescriptorSupport
{
  public static final String TYPE_ID = "repository.apt.rebuild-metadata.automatic";

  private static final String NAME = "Automated APT repository Metadata Rebuild";

  /**
   * Constructs a task descriptor for automated APT metadata rebuilds.
   */
  protected AutomatedAptMetadataRebuildTaskDescriptor() {
    super(
        TYPE_ID,
        AutomatedAptMetadataRebuildTask.class,
        NAME,
        NOT_VISIBLE,
        NOT_EXPOSED,
        REQUEST_RECOVERY,
        new RepositoryCombobox(
            RepositoryTaskSupport.REPOSITORY_NAME_FIELD_ID,
            "Repository",
            "Select the APT repository to rebuild metadata for",
            true).includingAnyOfFormats(AptFormat.NAME).includingAnyOfTypes(ProxyType.NAME, HostedType.NAME));
  }
}
