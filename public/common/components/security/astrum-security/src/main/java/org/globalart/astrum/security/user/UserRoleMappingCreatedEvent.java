
package org.globalart.astrum.security.user;

import java.util.Set;

/**
 * Emitted when a User role-mapping has been created.
 *
 * @since 3.1
 */
public class UserRoleMappingCreatedEvent
    extends UserRoleMappingEvent
{
  public UserRoleMappingCreatedEvent(
      final String userId,
      final String userSource,
      final Set<String> roles)
  {
    super(userId, userSource, roles);
  }
}
