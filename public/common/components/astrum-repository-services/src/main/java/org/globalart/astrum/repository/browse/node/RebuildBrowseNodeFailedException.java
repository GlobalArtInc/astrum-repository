
package org.globalart.astrum.repository.browse.node;

/**
 * For when rebuilding of browse nodes fails.
 *
 * @since 3.22
 */
public class RebuildBrowseNodeFailedException
    extends Exception
{
  public RebuildBrowseNodeFailedException(final String message, final Throwable cause) {
    super(message, cause);
  }
}
