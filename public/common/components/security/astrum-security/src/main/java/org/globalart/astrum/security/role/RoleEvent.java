
package org.globalart.astrum.security.role;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * {@link Role} event.
 *
 * @since 3.1
 */
public abstract class RoleEvent
{
  private final Role role;

  public RoleEvent(final Role role) {
    this.role = checkNotNull(role);
  }

  public Role getRole() {
    return role;
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "role=" + role +
        '}';
  }
}
