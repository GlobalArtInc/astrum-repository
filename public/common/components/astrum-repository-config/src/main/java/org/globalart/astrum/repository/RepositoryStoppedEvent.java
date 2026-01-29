
package org.globalart.astrum.repository;

/**
 * Emitted when a repository has been stopped.
 *
 * @since 3.0
 */
public class RepositoryStoppedEvent
    extends RepositoryEvent
{
  public RepositoryStoppedEvent(final Repository repository) {
    super(repository);
  }
}
