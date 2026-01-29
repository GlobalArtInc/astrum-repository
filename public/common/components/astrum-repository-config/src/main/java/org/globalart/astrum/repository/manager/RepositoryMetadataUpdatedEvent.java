
package org.globalart.astrum.repository.manager;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.RepositoryEvent;

/**
 * Emitted when a repository's metadata attributes have been updated.
 *
 * @since 3.6.1
 */
public class RepositoryMetadataUpdatedEvent
    extends RepositoryEvent
{
  public RepositoryMetadataUpdatedEvent(final Repository repository) {
    super(repository);
  }
}
