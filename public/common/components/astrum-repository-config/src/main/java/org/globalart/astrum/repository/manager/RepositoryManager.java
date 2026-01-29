
package org.globalart.astrum.repository.manager;

import java.util.Collection;
import java.util.Optional;
import java.util.Set;
import java.util.stream.Stream;

import javax.annotation.Nullable;

import org.globalart.astrum.common.lifecycle.Lifecycle;
import org.globalart.astrum.datastore.DataStoreUsageChecker;
import org.globalart.astrum.repository.Recipe;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.config.Configuration;

/**
 * Repository manager.
 *
 * @since 3.0
 */
public interface RepositoryManager
    extends DataStoreUsageChecker, Lifecycle
{
  Iterable<Repository> browse();

  /**
   * @since 3.6.1
   */
  Iterable<Repository> browseForBlobStore(String blobStoreId);

  boolean exists(String name);

  @Nullable
  Repository get(String name);

  /**
   * Returns a stream of the configurations for repositories registered with the manager. The Configuration objects
   * should not be modified. The associated repositories may not be started if invoked during startup.
   */
  Stream<Configuration> getConfigurations();

  @Nullable
  Repository softGet(String name);

  Optional<Configuration> retrieveConfigurationByName(String name);

  Repository create(Configuration configuration) throws Exception;

  Repository update(Configuration configuration) throws Exception;

  void delete(String name) throws Exception;

  boolean isBlobstoreUsed(String blobStoreName);

  long blobstoreUsageCount(String blobStoreName);

  /**
   * Returns a set of group repositories who directly or transitively contain the specified repository
   */
  Set<String> findContainingGroups(String repositoryName);

  Stream<Repository> browseForCleanupPolicy(final String cleanupPolicyName);

  Collection<Recipe> getAllSupportedRecipes();

  /**
   * Create a new {@link Configuration} instance.
   *
   * @since 3.21
   */
  Configuration newConfiguration();

  int count();
}
