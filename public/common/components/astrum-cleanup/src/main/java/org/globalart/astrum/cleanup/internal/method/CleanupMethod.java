
package org.globalart.astrum.cleanup.internal.method;

import java.util.function.BooleanSupplier;
import java.util.stream.Stream;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.task.DeletionProgress;

/**
 * Cleans up components.
 *
 * @since 3.14
 */
public interface CleanupMethod
{
  /**
   * Cleans up the given list of components.
   *
   * @param repository - the repository containing the components to cleanup
   * @param components - what to cleanup (delete / move)
   * @param cancelledCheck - allows the cleanup to be stopped by the caller before finishing iterating the components
   * @return the number of components cleaned up
   */
  DeletionProgress run(
      Repository repository,
      Stream<FluentComponent> components,
      BooleanSupplier cancelledCheck);
}
