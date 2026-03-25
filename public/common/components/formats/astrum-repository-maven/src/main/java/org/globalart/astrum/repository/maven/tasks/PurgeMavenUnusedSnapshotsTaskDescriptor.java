
package org.globalart.astrum.repository.maven.tasks;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.formfields.FormField;
import org.globalart.astrum.formfields.NumberTextFormField;
import org.globalart.astrum.formfields.RepositoryCombobox;
import org.globalart.astrum.repository.maven.PurgeUnusedSnapshotsFacet;
import org.globalart.astrum.repository.maven.VersionPolicy;
import org.globalart.astrum.scheduling.TaskDescriptorSupport;

import static org.globalart.astrum.repository.RepositoryTaskSupport.REPOSITORY_NAME_FIELD_ID;
import static org.globalart.astrum.repository.maven.tasks.PurgeMavenUnusedSnapshotsTask.LAST_USED_FIELD_ID;
import org.springframework.stereotype.Component;

/**
 * Task descriptor for {@link PurgeMavenUnusedSnapshotsTask}.
 *
 * @since 3.0
 */
@AvailabilityVersion(from = "1.0")
@Component
public class PurgeMavenUnusedSnapshotsTaskDescriptor
    extends TaskDescriptorSupport
{
  public static final String TASK_NAME = "Maven - Delete unused SNAPSHOT";

  public static final String TYPE_ID = "repository.maven.purge-unused-snapshots";

  public static final Number LAST_USED_INIT_VALUE = 1;

  public static final Number LAST_USED_MIN_VALUE = 1;

  public PurgeMavenUnusedSnapshotsTaskDescriptor() {
    super(TYPE_ID,
        PurgeMavenUnusedSnapshotsTask.class,
        TASK_NAME,
        VISIBLE,
        EXPOSED,
        new RepositoryCombobox(
            REPOSITORY_NAME_FIELD_ID,
            "Repository",
            "Select the repository to delete unused snapshot versions from",
            FormField.MANDATORY).includingAnyOfFacets(PurgeUnusedSnapshotsFacet.class)
                .excludingAnyOfVersionPolicies(VersionPolicy.RELEASE.name())
                .includeAnEntryForAllRepositories(),
        new NumberTextFormField(
            LAST_USED_FIELD_ID,
            "Last used in days",
            "Delete all snapshots that were last used before given number of days",
            FormField.MANDATORY).withInitialValue(LAST_USED_INIT_VALUE).withMinimumValue(LAST_USED_MIN_VALUE));
  }
}
