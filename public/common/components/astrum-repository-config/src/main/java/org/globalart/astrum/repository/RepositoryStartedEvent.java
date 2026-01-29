
package org.globalart.astrum.repository;

/**
 * Emitted when a repository has been started.
 *
 * @since 3.0
 */
public class RepositoryStartedEvent
    extends RepositoryEvent
{
  public RepositoryStartedEvent(final Repository repository) {
    super(repository);
  }
}
