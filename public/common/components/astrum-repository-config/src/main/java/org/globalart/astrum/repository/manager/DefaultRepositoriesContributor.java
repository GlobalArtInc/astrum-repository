
package org.globalart.astrum.repository.manager;

import java.util.List;

import org.globalart.astrum.repository.config.Configuration;

/**
 * Provides configurations for repositories that should be provisioned by default on first system startup.
 * 
 * @since 3.0
 */
public interface DefaultRepositoriesContributor
{
  /**
   * Provides Configurations that should be be initially provisioned.
   */
  List<Configuration> getRepositoryConfigurations();
}
