
package org.globalart.astrum.repository.manager;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.RepositoryEvent;
import org.globalart.astrum.repository.config.Configuration;

/**
 * Emitted when a repository has been updated.
 *
 * @since 3.0
 */
public class RepositoryUpdatedEvent
    extends RepositoryEvent
{
  private final Configuration oldConfiguration;

  public RepositoryUpdatedEvent(final Repository repository, final Configuration oldConfiguration) {
    super(repository);
    this.oldConfiguration = oldConfiguration;
  }

  /**
   * The previous configuration of the Repository.
   *
   * @since 3.21
   */
  public Configuration getOldConfiguration() {
    return oldConfiguration;
  }
}
