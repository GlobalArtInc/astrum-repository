
package org.globalart.astrum.repository;

/**
 * An exception thrown when the write policy of a repository prevents writing to a particular path
 */
public class RedeployDisabledException
    extends IllegalOperationException
{
  public RedeployDisabledException(final Repository repository, final String path, final String reason) {
    super(repository.getName() + path + " - " + reason);
  }
}
