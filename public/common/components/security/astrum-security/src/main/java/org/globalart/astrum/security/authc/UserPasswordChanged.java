
package org.globalart.astrum.security.authc;

/**
 * An event fired when the user's password has changed.
 *
 * @since 3.13
 */
public class UserPasswordChanged
{
  private final String userId;

  private final boolean clearCache;

  public UserPasswordChanged(final String userId) {
    this(userId, true);
  }

  public UserPasswordChanged(final String userId, final boolean clearCache) {
    this.userId = userId;
    this.clearCache = clearCache;
  }

  public String getUserId() {
    return userId;
  }

  public boolean isClearCache() {
    return clearCache;
  }
}
