
package org.globalart.astrum.repository.maven.tasks;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.formfields.CheckboxFormField;
import org.globalart.astrum.formfields.NumberTextFormField;
import org.globalart.astrum.formfields.RepositoryCombobox;
import org.globalart.astrum.repository.maven.RemoveSnapshotsFacet;
import org.globalart.astrum.repository.maven.VersionPolicy;
import org.globalart.astrum.scheduling.TaskDescriptorSupport;
import org.springframework.stereotype.Component;

/**
 * Configuration definition for {@link RemoveSnapshotsTask}
 * 
 * @since 3.0
 */
@AvailabilityVersion(from = "1.0")
@Component
public class RemoveSnapshotsTaskDescriptor
    extends TaskDescriptorSupport
{
  public static final String TYPE_ID = "repository.maven.remove-snapshots";

  public static final String REPOSITORY_NAME_FIELD_ID = "repositoryName";

  public static final String MINIMUM_SNAPSHOT_RETAINED_COUNT = "minimumRetained";

  public static final String SNAPSHOT_RETENTION_DAYS = "snapshotRetentionDays";

  public static final String REMOVE_IF_RELEASED = "removeIfReleased";

  public static final String GRACE_PERIOD = "gracePeriodInDays";

  public RemoveSnapshotsTaskDescriptor() {
    super(TYPE_ID,
        RemoveSnapshotsTask.class,
        "Maven - Delete SNAPSHOT",
        VISIBLE,
        EXPOSED,
        new RepositoryCombobox(REPOSITORY_NAME_FIELD_ID,
            "Repository",
            "Select the Maven repository or repository group to remove snapshots from.",
            true).includingAnyOfFacets(RemoveSnapshotsFacet.class)
                .excludingAnyOfVersionPolicies(VersionPolicy.RELEASE.name())
                .includeAnEntryForAllRepositories(),
        new NumberTextFormField(MINIMUM_SNAPSHOT_RETAINED_COUNT,
            "Minimum snapshot count",
            "Minimum number of snapshots to keep for one GAV.",
            true).withInitialValue(1).withMinimumValue(-1),
        new NumberTextFormField(SNAPSHOT_RETENTION_DAYS,
            "Snapshot retention (days)",
            "Delete all snapshots older than this, provided we still keep the minimum number specified.",
            true).withInitialValue(30).withMinimumValue(0),
        new CheckboxFormField(REMOVE_IF_RELEASED,
            "Remove if released",
            "Delete all snapshots that have a corresponding release", false),
        new NumberTextFormField(GRACE_PERIOD,
            "Grace period after release (days)",
            "The grace period during which snapshots with an associated release will not be deleted.",
            false).withMinimumValue(0));
  }
}
