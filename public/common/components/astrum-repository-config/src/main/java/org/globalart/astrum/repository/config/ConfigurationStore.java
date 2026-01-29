
package org.globalart.astrum.repository.config;

import java.util.Collection;
import java.util.List;
import java.util.Set;

import org.globalart.astrum.common.lifecycle.Lifecycle;

/**
 * {@link Configuration} store.
 *
 * @since 3.0
 */
public interface ConfigurationStore
    extends Lifecycle
{
  List<Configuration> list();

  void create(Configuration configuration);

  void update(Configuration configuration);

  void delete(Configuration configuration);

  /**
   * Create a new {@link Configuration} instance.
   *
   * @since 3.21
   */
  Configuration newConfiguration();

  Collection<Configuration> readByNames(Set<String> repositoryNames);

  /**
   * Checks whether the database contains an entry with the provided name.<br>
   *
   * Note: Underlying implementations may return {@code false} if unsupported.
   */
  boolean exists(String repositoryName);

  /**
   * Read all repository configuration by given {@code recipeName}
   * 
   * @param recipeName recipe name for collecting configurations
   * @return configurations filtered by {@code recipeName}
   */
  Collection<Configuration> readByRecipe(String recipeName);
}
