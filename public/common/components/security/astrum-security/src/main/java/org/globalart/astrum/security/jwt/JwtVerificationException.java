
package org.globalart.astrum.security.jwt;

/**
 * Exception with details about a JWT verification attempt.
 *
 * @since 3.38
 */
public class JwtVerificationException
    extends Exception
{
  private static final long serialVersionUID = 8292613157279521051L;

  public JwtVerificationException(final String reason) {
    super(reason);
  }
}
