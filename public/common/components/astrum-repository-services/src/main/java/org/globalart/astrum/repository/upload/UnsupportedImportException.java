
package org.globalart.astrum.repository.upload;

/**
 * Exception thrown when a repository format does not support the requested import operation.
 *
 * @since 3.31
 */
public class UnsupportedImportException
    extends UnsupportedOperationException
{
  public UnsupportedImportException(String message) {
    super(message);
  }

  public UnsupportedImportException(String message, Throwable cause) {
    super(message, cause);
  }
}
