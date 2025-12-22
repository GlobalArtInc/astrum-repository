
package org.globalart.astrum.security.user;

/**
 * Thrown when a user could not be found due to a temporary condition.
 *
 * For example when an LDAP server is unavailable.
 * Repeating the operation may succeed in the future without any intervention by the application.
 */
public class UserNotFoundTransientException
    extends UserNotFoundException
{
  private static final long serialVersionUID = 7565547428483146620L;

  public UserNotFoundTransientException(final String userId, final String message, final Throwable cause) {
    super(userId, message, cause);
  }
}
