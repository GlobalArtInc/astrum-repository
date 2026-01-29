
package org.globalart.astrum.repository;

/**
 * Emitted when a repository has been destroyed.
 *
 * @since 3.0
 */
public class RepositoryDestroyedEvent
    extends RepositoryEvent
{
  public RepositoryDestroyedEvent(final Repository repository) {
    super(repository);
  }
}
