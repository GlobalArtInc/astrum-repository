
package org.globalart.astrum.security.user;

/**
 * Thrown if the password isn't correct on reset password.
 */
public class InvalidCredentialsException
    extends Exception
{
  private static final long serialVersionUID = 294536984704055394L;

  public InvalidCredentialsException() {
    super("Invalid credentials");
  }
}
