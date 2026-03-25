
package org.globalart.astrum.repository.maven.tasks;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.formfields.RepositoryCombobox;
import org.globalart.astrum.repository.maven.internal.Maven2Format;
import org.globalart.astrum.scheduling.TaskDescriptorSupport;
import org.springframework.stereotype.Component;

/**
 * Task descriptor for {@link PublishMavenIndexTask}.
 *
 * @since 3.0
 */
@AvailabilityVersion(from = "1.0")
@Component
public class PublishMavenIndexTaskDescriptor
    extends TaskDescriptorSupport
{
  public static final String TYPE_ID = "repository.maven.publish-dotindex";

  public static final String REPOSITORY_NAME_FIELD_ID = "repositoryName";

  public PublishMavenIndexTaskDescriptor() {
    super(TYPE_ID,
        PublishMavenIndexTask.class,
        "Maven - Publish Maven Indexer files",
        VISIBLE,
        EXPOSED,
        new RepositoryCombobox(
            REPOSITORY_NAME_FIELD_ID,
            "Repository",
            "Select the Maven repository to publish indexer files for",
            true).includingAnyOfFormats(Maven2Format.NAME).includeAnEntryForAllRepositories());
  }
}
