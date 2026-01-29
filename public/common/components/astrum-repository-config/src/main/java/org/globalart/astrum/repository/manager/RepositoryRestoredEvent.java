
package org.globalart.astrum.repository.manager;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.RepositoryEvent;

/**
 * Emitted when a repository has been restored.
 *
 * @since 3.0
 */
public class RepositoryRestoredEvent
    extends RepositoryEvent
{
  public RepositoryRestoredEvent(final Repository repository) {
    super(repository);
  }
}
