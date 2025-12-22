
package org.globalart.astrum.security.user;

/**
 * Thrown when {@link UserManager} could not be found.
 */
public class NoSuchUserManagerException
    extends Exception
{
  private static final long serialVersionUID = -2561129270233203244L;

  public NoSuchUserManagerException(final String source) {
    this("User-manager not found", source);
  }

  public NoSuchUserManagerException(final String message, final String source) {
    super(message + ": " + source);
  }
}
