
package org.globalart.astrum.security.usertoken.event;

/**
 * Event fired when an administrator reads user token metadata.
 */
public class UserTokenAdminReadEvent
    extends UserTokenEvent
{
  private final String targetUserId;

  private final String targetRealm;

  private final String adminUserId;

  private final String adminRealm;

  public UserTokenAdminReadEvent(
      final String targetUserId,
      final String targetRealm,
      final String adminUserId,
      final String adminRealm)
  {
    super(UserTokenEventTypes.ADMIN_READ);
    this.targetUserId = targetUserId;
    this.targetRealm = targetRealm;
    this.adminUserId = adminUserId;
    this.adminRealm = adminRealm;
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
}
