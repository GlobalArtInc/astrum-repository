
package org.globalart.astrum.blobstore.api;

/**
 * @since 3.34
 */
public class RepositoryDoesNotExistException
    extends Exception
{
  private static final String MESSAGE = "Repository does not exist";

  public RepositoryDoesNotExistException() {
    super(MESSAGE);
  }
}
