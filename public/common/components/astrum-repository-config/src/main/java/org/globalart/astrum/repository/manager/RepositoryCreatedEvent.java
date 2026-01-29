
package org.globalart.astrum.repository.manager;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.RepositoryEvent;

/**
 * Emitted when a repository has been created.
 *
 * @since 3.0
 */
public class RepositoryCreatedEvent
    extends RepositoryEvent
{
  public RepositoryCreatedEvent(final Repository repository) {
    super(repository);
  }
}
