
package org.globalart.astrum.security.user;

/**
 * Emitted when a {@link User} has been created.
 *
 * @since 3.1
 */
public class UserCreatedEvent
    extends UserEvent
{
  public UserCreatedEvent(final User user) {
    super(user);
  }
}
