
package org.globalart.astrum.security.usertoken.event;

/**
 * Event fired when an administrator lists user tokens.
 */
public class UserTokenAdminListedEvent
    extends UserTokenEvent
{
  private final String adminUserId;

  private final String adminRealm;

  private final String realmFilter;

  private final String userIdFilter;

  private final boolean includeExpired;

  private final int resultCount;

  public UserTokenAdminListedEvent(
      final String adminUserId,
      final String adminRealm,
      final String realmFilter,
      final String userIdFilter,
      final boolean includeExpired,
      final int resultCount)
  {
    super(UserTokenEventTypes.ADMIN_LISTED);
    this.adminUserId = adminUserId;
    this.adminRealm = adminRealm;
    this.realmFilter = realmFilter;
    this.userIdFilter = userIdFilter;
    this.includeExpired = includeExpired;
    this.resultCount = resultCount;
  }

  public String getAdminUserId() {
    return adminUserId;
  }

  public String getAdminRealm() {
    return adminRealm;
  }

  public String getRealmFilter() {
    return realmFilter;
  }

  public String getUserIdFilter() {
    return userIdFilter;
  }

  public boolean isIncludeExpired() {
    return includeExpired;
  }

  public int getResultCount() {
    return resultCount;
  }
}
