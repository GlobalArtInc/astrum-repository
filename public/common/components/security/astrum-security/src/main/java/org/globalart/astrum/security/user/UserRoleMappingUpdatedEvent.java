
package org.globalart.astrum.security.user;

import java.util.Set;

/**
 * Emitted when a User role-mapping has been updated.
 *
 * @since 3.1
 */
public class UserRoleMappingUpdatedEvent
    extends UserRoleMappingEvent
{
  public UserRoleMappingUpdatedEvent(
      final String userId,
      final String userSource,
      final Set<String> roles)
  {
    super(userId, userSource, roles);
  }
}
