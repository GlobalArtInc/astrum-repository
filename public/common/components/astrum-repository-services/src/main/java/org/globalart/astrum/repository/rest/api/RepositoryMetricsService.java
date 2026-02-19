
package org.globalart.astrum.repository.rest.api;

import java.util.List;
import java.util.Optional;

/**
 * Service for handling metrics for Repositories.
 */
public interface RepositoryMetricsService
{
  /**
   * Retrieve metrics for the given repository name.
   */
  Optional<RepositoryMetricsDTO> get(final String repositoryName);

  /**
   * Retrieve metrics for all repositories.
   */
  List<RepositoryMetricsDTO> list();

  /**
   * Runs the repo sizes update.
   */
  void runUpdate();
}
