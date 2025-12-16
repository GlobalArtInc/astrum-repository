
package org.globalart.astrum.security;

import org.globalart.astrum.common.event.EventWithSource;

/**
 * An event fired when a user is removed from the system, so cached principals can be expired.
 */
public class UserPrincipalsExpired
    extends EventWithSource
{
  private final String userId;

  private final String source;

  /**
   * Applies to any cached user principals that have the given userId and UserManager source.
   *
   * @param userId The removed user's id
   * @param source The UserManager source
   */
  public UserPrincipalsExpired(final String userId, final String source) {
    this.userId = userId;
    this.source = source;
  }

  /**
   * Applies to all cached user principals that have an invalid userId or UserManager source.
   */
  public UserPrincipalsExpired() {
    this(null, null);
  }

  public String getUserId() {
    return userId;
  }

  public String getSource() {
    return source;
  }
}
