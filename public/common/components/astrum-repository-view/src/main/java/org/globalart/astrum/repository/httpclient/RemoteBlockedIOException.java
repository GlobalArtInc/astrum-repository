
package org.globalart.astrum.repository.httpclient;

import java.io.IOException;

/**
 * Thrown when a remote is manually or automatically blocked.
 *
 * @since 3.3
 */
public class RemoteBlockedIOException
    extends IOException
{
  public RemoteBlockedIOException(final String message) {
    super(message);
  }
}
