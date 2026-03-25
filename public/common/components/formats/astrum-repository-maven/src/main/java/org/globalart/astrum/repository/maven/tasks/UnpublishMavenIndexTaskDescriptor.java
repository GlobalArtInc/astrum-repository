
package org.globalart.astrum.repository.maven.tasks;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.formfields.RepositoryCombobox;
import org.globalart.astrum.repository.maven.internal.Maven2Format;
import org.globalart.astrum.scheduling.TaskDescriptorSupport;
import org.springframework.stereotype.Component;

/**
 * Task descriptor for {@link UnpublishMavenIndexTask}.
 *
 * @since 3.0
 */
@AvailabilityVersion(from = "1.0")
@Component
public class UnpublishMavenIndexTaskDescriptor
    extends TaskDescriptorSupport
{
  public static final String TYPE_ID = "repository.maven.unpublish-dotindex";

  public static final String REPOSITORY_NAME_FIELD_ID = "repositoryName";

  public UnpublishMavenIndexTaskDescriptor() {
    super(TYPE_ID,
        UnpublishMavenIndexTask.class,
        "Maven - Unpublish Maven Indexer files",
        VISIBLE,
        EXPOSED,
        new RepositoryCombobox(
            REPOSITORY_NAME_FIELD_ID,
            "Repository",
            "Select the Maven repository to remove indexes for",
            true).includingAnyOfFormats(Maven2Format.NAME).includeAnEntryForAllRepositories());
  }
}
