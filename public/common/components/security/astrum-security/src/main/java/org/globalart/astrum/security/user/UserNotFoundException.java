
package org.globalart.astrum.security.user;

/**
 * Thrown when a user could not be found.
 */
public class UserNotFoundException
    extends Exception
{
  private static final long serialVersionUID = -177760017345640029L;

  public UserNotFoundException(final String userId, final String message, final Throwable cause) {
    super("User not found: " + userId + "; " + message, cause);
  }

  public UserNotFoundException(final String userId) {
    super("User not found: " + userId);
  }
}
