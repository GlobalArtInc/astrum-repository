
package org.globalart.astrum.security.authz;

/**
 * Thrown when an {@link AuthorizationManager} could not be found.
 */
public class NoSuchAuthorizationManagerException
    extends Exception
{
  private static final long serialVersionUID = -9130834235862218360L;

  public NoSuchAuthorizationManagerException(final String source) {
    super("Authorization-manager with source '" + source + "' could not be found");
  }
}
