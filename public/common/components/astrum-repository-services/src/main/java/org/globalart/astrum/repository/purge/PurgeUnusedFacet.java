
package org.globalart.astrum.repository.purge;

import org.globalart.astrum.repository.Facet;

/**
 * Find & delete components and assets that were not used/accessed for a number of days.
 *
 * @since 3.0
 */
@Facet.Exposed
public interface PurgeUnusedFacet
    extends Facet
{
  /**
   * Find & delete components and assets that were not used/accessed for a number of days.
   *
   * @param numberOfDays number of days from the moment the method is invoked. Must be > 0.
   */
  void purgeUnused(int numberOfDays);
}
