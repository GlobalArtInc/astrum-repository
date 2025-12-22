
package org.globalart.astrum.security.user;

/**
 * Emitted when a {@link User} has been updated.
 *
 * @since 3.1
 */
public class UserUpdatedEvent
    extends UserEvent
{
  public UserUpdatedEvent(final User user) {
    super(user);
  }
}
