package org.globalart.astrum.oidc.internal;

public class OidcException
    extends Exception
{
  public OidcException(final String message) {
    super(message);
  }

  public OidcException(final String message, final Throwable cause) {
    super(message, cause);
  }
}
