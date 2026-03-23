
package org.globalart.astrum.repository.maven;

import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.maven.tasks.RemoveSnapshotsConfig;

/**
 * Facet handling the removal of snapshots from a repository.
 *
 * @since 3.0
 */
@Facet.Exposed
public interface RemoveSnapshotsFacet
    extends Facet
{

  /**
   * Delete snapshots matching this configuration, and update associated metadata in the repository.
   *
   * @param removeSnapshotsConfig {@link RemoveSnapshotsConfig}
   */
  void removeSnapshots(RemoveSnapshotsConfig removeSnapshotsConfig);
}
