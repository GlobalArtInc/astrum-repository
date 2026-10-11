package org.globalart.astrum.accesstoken.internal;

/**
 * A request about access tokens that cannot be fulfilled; the message is shown to the user.
 */
public class AccessTokenException
    extends RuntimeException
{
  public AccessTokenException(final String message) {
    super(message);
  }
}
