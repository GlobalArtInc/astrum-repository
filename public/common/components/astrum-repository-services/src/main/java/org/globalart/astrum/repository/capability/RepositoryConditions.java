
package org.globalart.astrum.repository.capability;

import java.util.function.Supplier;

import org.globalart.astrum.capability.Condition;

/**
 * Factory of {@link Condition}s related to repositories.
 *
 * @since 3.0
 */
public interface RepositoryConditions
{
  /**
   * Creates a new condition that is satisfied when a repository is in service.
   *
   * @param repositoryName getter for repository name (usually condition specific property)
   * @return created condition
   */
  Condition repositoryIsOnline(Supplier<String> repositoryName);

  /**
   * Creates a new condition that is satisfied when a repository exists.
   *
   * @param repositoryName getter for repository name (usually condition specific property)
   * @return created condition
   */
  Condition repositoryExists(Supplier<String> repositoryName);
}
