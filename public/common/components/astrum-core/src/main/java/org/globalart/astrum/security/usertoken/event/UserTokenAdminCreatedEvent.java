
package org.globalart.astrum.security.usertoken.event;

/**
 * Event fired when an administrator creates a user token for another user.
 */
public class UserTokenAdminCreatedEvent
    extends UserTokenEvent
{
  private final String targetUserId;

  private final String targetRealm;

  private final String adminUserId;

  private final String adminRealm;

  private final String nameCode;

  public UserTokenAdminCreatedEvent(
      final String targetUserId,
      final String targetRealm,
      final String adminUserId,
      final String adminRealm,
      final String nameCode)
  {
    super(UserTokenEventTypes.ADMIN_CREATED);
    this.targetUserId = targetUserId;
    this.targetRealm = targetRealm;
    this.adminUserId = adminUserId;
    this.adminRealm = adminRealm;
    this.nameCode = nameCode;
  }

  public String getTargetUserId() {
    return targetUserId;
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

  public String getNameCode() {
    return nameCode;
  }
}
