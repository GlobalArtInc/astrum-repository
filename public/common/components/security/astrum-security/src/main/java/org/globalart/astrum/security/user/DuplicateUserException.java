
package org.globalart.astrum.security.user;

/**
 * @since 3.21
 */
public class DuplicateUserException
    extends RuntimeException
{
  private static final String ERROR_TEXT = "User %s already exists.";

  public DuplicateUserException(final String userId, final Throwable cause) {
    super(String.format(ERROR_TEXT, userId), cause);
  }

  public DuplicateUserException(final String userId) {
    this(userId, null);
  }
}
