
package org.globalart.astrum.security.user;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * {@link User} event.
 *
 * @since 3.1
 */
public abstract class UserEvent
{
  private final User user;

  public UserEvent(final User user) {
    this.user = checkNotNull(user);
  }

  public User getUser() {
    return user;
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "user=" + user +
        '}';
  }
}
