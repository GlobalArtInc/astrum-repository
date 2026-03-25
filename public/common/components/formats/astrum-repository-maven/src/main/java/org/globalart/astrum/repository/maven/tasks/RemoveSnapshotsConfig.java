
package org.globalart.astrum.repository.maven.tasks;

/**
 * Configuration for {@link org.globalart.astrum.repository.maven.tasks.RemoveSnapshotsTask}.
 *
 * @since 3.0
 */
public class RemoveSnapshotsConfig
{
  private final int minimumRetained;

  private final int snapshotRetentionDays;

  private final boolean removeIfReleased;

  private final int gracePeriod;

  public RemoveSnapshotsConfig(
      final int minimumRetained,
      final int snapshotRetentionDays,
      final boolean removeIfReleased,
      final int gracePeriod)
  {
    this.minimumRetained = minimumRetained;
    this.snapshotRetentionDays = snapshotRetentionDays;
    this.removeIfReleased = removeIfReleased;
    this.gracePeriod = gracePeriod;
  }

  /**
   * The minimum number of snapshots to keep.
   */
  public int getMinimumRetained() {
    return minimumRetained;
  }

  /**
   * Snapshots older than this will be candidates for removal.
   */
  public int getSnapshotRetentionDays() {
    return snapshotRetentionDays;
  }

  /**
   * Whether or not to delete snapshots if a release version of the same artifact is available.
   */
  public boolean getRemoveIfReleased() {
    return removeIfReleased;
  }

  /**
   * An optional period to keep snapshots around, even if a release version of the same artifact is available.
   */
  public int getGracePeriod() {
    return gracePeriod;
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "minimumRetained=" + minimumRetained +
        ", snapshotRetentionDays=" + snapshotRetentionDays +
        ", removeIfReleased=" + removeIfReleased +
        ", gracePeriod=" + gracePeriod +
        "}";
  }
}
