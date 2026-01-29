
package org.globalart.astrum.repository;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Repository event.
 *
 * @since 3.0
 */
public abstract class RepositoryEvent
{
  private final Repository repository;

  public RepositoryEvent(final Repository repository) {
    this.repository = checkNotNull(repository);
  }

  public Repository getRepository() {
    return repository;
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "repository=" + repository +
        '}';
  }
}
