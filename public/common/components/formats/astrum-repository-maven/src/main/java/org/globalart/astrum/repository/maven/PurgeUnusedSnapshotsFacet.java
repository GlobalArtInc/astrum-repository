
package org.globalart.astrum.repository.maven;

import org.globalart.astrum.repository.Facet;

/**
 * Facet for purging unused Maven snapshots.
 *
 * @since 3.0
 */
@Facet.Exposed
public interface PurgeUnusedSnapshotsFacet
    extends Facet
{
  /**
   * Purges snapshots that were not used/accessed for a number of days.
   *
   * @param numberOfDays number of days from the moment the method is invoked. Must be > 0.
   */
  void purgeUnusedSnapshots(int numberOfDays);
}
