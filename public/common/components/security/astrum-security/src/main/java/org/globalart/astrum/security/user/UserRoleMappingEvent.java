
package org.globalart.astrum.security.user;

import java.util.Set;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * User role-mapping event.
 *
 * @since 3.1
 */
public abstract class UserRoleMappingEvent
{
  // NOTE: there is no high-level object representation of user role-mapping, and there is no User object
  // NOTE: ... accessible where the mapping is managed

  private final String userId;

  private final String userSource;

  private final Set<String> roles;

  public UserRoleMappingEvent(
      final String userId,
      final String userSource,
      final Set<String> roles)
  {
    this.userId = checkNotNull(userId);
    this.userSource = checkNotNull(userSource);
    this.roles = checkNotNull(roles);
  }

  public String getUserId() {
    return userId;
  }

  public String getUserSource() {
    return userSource;
  }

  public Set<String> getRoles() {
    return roles;
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "userId='" + userId + '\'' +
        ", userSource='" + userSource + '\'' +
        ", roles=" + roles +
        '}';
  }
}
