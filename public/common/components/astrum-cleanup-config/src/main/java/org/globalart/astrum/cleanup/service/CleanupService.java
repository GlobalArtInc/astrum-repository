
package org.globalart.astrum.cleanup.service;

import java.util.function.BooleanSupplier;

import org.globalart.astrum.repository.Repository;

/**
 * Runs cleanup by looping through all repositories and evaluating their cleanup policies.
 *
 * @since 3.14
 */
public interface CleanupService
{
  void cleanup(BooleanSupplier cancelledCheck);

  long cleanupRepository(Repository repository, BooleanSupplier cancelledCheck);

  long dryRunCount(Repository repository);
}
