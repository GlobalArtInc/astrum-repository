
package org.globalart.astrum.repository.rest.api;

import java.util.List;
import java.util.Optional;

import javax.annotation.Nonnull;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.config.Configuration;

/**
 * A repository manager which limits access to repositories based on the current user's permissions.
 *
 * @since 3.20
 */
public interface AuthorizingRepositoryManager
{
  void create(@Nonnull Configuration configuration) throws Exception;

  boolean update(@Nonnull Configuration configuration) throws Exception;

  boolean delete(@Nonnull String name) throws Exception;

  /**
   * Returns the repositories which the user has an administrative read privilege.
   */
  List<Repository> getRepositoriesWithAdmin();

  /**
   * Returns the repository if the user has administrative read privileges.
   *
   * @since 3.27
   */
  Optional<Repository> getRepositoryWithAdmin(String repositoryName);

  /**
   * Trigger rebuild index task for given repository.
   *
   * @throws RepositoryNotFoundException if repository does not exists
   * @throws IncompatibleRepositoryException if is not hosted or proxy type
   */
  @SuppressWarnings("squid:S1160") // suppress warning about two checked exceptions
  void rebuildSearchIndex(@Nonnull String name) throws RepositoryNotFoundException, IncompatibleRepositoryException;

  /**
   * Invalidate cache of a given repository.
   *
   * @throws RepositoryNotFoundException if repository does not exists
   * @throws IllegalStateException if is not proxy or group type
   */
  @SuppressWarnings("squid:S1160") // suppress warning about two checked exceptions
  void invalidateCache(@Nonnull String name) throws RepositoryNotFoundException, IncompatibleRepositoryException;
}
