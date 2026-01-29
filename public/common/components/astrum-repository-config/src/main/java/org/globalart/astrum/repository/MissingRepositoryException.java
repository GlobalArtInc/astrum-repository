
package org.globalart.astrum.repository;

/**
 * Thrown when a requested repository does not exist or is not yet available.
 * This commonly occurs during event replication race conditions where an event
 * references a repository that has been deleted or not yet created.
 */
public class MissingRepositoryException
    extends IllegalStateException
{
  private final String repositoryName;

  public MissingRepositoryException(final String repositoryName) {
    super(String.format("Missing repository: %s", repositoryName));
    this.repositoryName = repositoryName;
  }

  public String getRepositoryName() {
    return repositoryName;
  }
}
