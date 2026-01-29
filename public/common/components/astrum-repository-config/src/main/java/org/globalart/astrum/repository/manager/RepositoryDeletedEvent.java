
package org.globalart.astrum.repository.manager;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.RepositoryEvent;

/**
 * Emitted when a repository has been deleted.
 *
 * @since 3.0
 */
public class RepositoryDeletedEvent
    extends RepositoryEvent
{
  public RepositoryDeletedEvent(final Repository repository) {
    super(repository);
  }
}
