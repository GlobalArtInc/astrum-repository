
package org.globalart.astrum.repository;

import javax.annotation.Nonnull;

import org.globalart.astrum.repository.config.Configuration;

/**
 * Repository recipe.
 *
 * @since 3.0
 */
public interface Recipe
{
  Type getType();

  Format getFormat();

  void apply(@Nonnull Repository repository) throws Exception;

  default void apply(@Nonnull Repository repository, Configuration configuration) throws Exception {
    apply(repository);
  }

  boolean isFeatureEnabled();
}
