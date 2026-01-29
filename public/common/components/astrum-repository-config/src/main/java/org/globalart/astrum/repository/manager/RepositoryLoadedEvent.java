
package org.globalart.astrum.repository.manager;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.RepositoryEvent;

/**
 * Emitted when a repository has been loaded.
 *
 * @since 3.0
 */
public class RepositoryLoadedEvent
    extends RepositoryEvent
{
  public RepositoryLoadedEvent(final Repository repository) {
    super(repository);
  }
}
