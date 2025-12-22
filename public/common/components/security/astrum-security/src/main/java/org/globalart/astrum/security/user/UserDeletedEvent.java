
package org.globalart.astrum.security.user;

/**
 * Emitted when a {@link User} has been deleted.
 *
 * @since 3.1
 */
public class UserDeletedEvent
    extends UserEvent
{
  public UserDeletedEvent(final User user) {
    super(user);
  }
}
