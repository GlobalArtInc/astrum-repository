
package org.globalart.astrum.cleanup.datastore.search.criteria;

import java.util.function.Predicate;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.Asset;

/**
 * An evaluator of a cleanup policy criteria which is only concerned with an Asset
 *
 * @since 3.38
 */
public interface AssetCleanupEvaluator
{
  /**
   * Creates a predicate which indicates whether the asset is eligible for cleanup.
   *
   * @param repository the source repository for the assets which will be fed into the predicate
   * @param value the value associated with the CleanupPolicy for use with this criteria
   * @return the predicate
   */
  Predicate<Asset> getPredicate(Repository repository, String value);
}
