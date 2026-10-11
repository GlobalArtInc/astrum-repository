package org.globalart.astrum.notifications.internal;

import java.net.ConnectException;

/**
 * The proxy of a channel did not let a connection through. The message is written for the administrator and never
 * contains the proxy password. Being a {@link ConnectException}, it is not retried by the HTTP client itself.
 */
public class ProxyConnectException
    extends ConnectException
{
  private final boolean retryable;

  public ProxyConnectException(final String message, final boolean retryable) {
    super(message);
    this.retryable = retryable;
  }

  /**
   * @return whether trying again later may succeed, unlike after a rejected password
   */
  public boolean isRetryable() {
    return retryable;
  }
}
