
package org.globalart.astrum.security.user;

import java.util.Collections;

/**
 * Emitted when a User role-mapping has been deleted.
 *
 * @since 3.1
 */
public class UserRoleMappingDeletedEvent
    extends UserRoleMappingEvent
{
  public UserRoleMappingDeletedEvent(final String userId, final String userSource) {
    super(userId, userSource, Collections.emptySet());
  }
}
