
package org.globalart.astrum.cleanup.datastore.search.criteria;

import java.util.function.BiPredicate;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.Component;

/**
 * An evaluator for cleanup tasks that cares about the Component, or the all the assets.
 *
 * Note: If the criteria is interested in only a single Asset see AssetCleanupEvaluator.
 *
 * @since 3.38
 */
public interface ComponentCleanupEvaluator
{
  /**
   * Creates a predicate which indicates whether the component is eligible for cleanup.
   *
   * @param repository the source repository for the assets which will be fed into the predicate
   * @param value the value associated with the CleanupPolicy for use with this criteria
   * @return the predicate
   */
  BiPredicate<Component, Iterable<Asset>> getPredicate(Repository repository, String value);
}
