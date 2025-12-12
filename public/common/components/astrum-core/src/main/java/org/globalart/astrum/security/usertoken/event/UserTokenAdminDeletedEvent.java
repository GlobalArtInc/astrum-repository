
package org.globalart.astrum.security.usertoken.event;

/**
 * Event fired when an administrator deletes a user token for another user.
 */
public class UserTokenAdminDeletedEvent
    extends UserTokenDeletedEvent
{
  private final String targetRealm;

  private final String adminUserId;

  private final String adminRealm;

  public UserTokenAdminDeletedEvent(
      final String targetUserId,
      final String targetRealm,
      final String adminUserId,
      final String adminRealm)
  {
    super(UserTokenEventTypes.ADMIN_DELETED, 1, targetUserId);
    this.targetRealm = targetRealm;
    this.adminUserId = adminUserId;
    this.adminRealm = adminRealm;
  }

  public String getTargetRealm() {
    return targetRealm;
  }

  public String getAdminUserId() {
    return adminUserId;
  }

  public String getAdminRealm() {
    return adminRealm;
  }
}
