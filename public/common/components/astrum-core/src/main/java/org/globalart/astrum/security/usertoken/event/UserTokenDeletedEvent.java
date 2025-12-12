
package org.globalart.astrum.security.usertoken.event;

public class UserTokenDeletedEvent
    extends UserTokenEvent
{
  private final int deleted;

  private final String username;

  public UserTokenDeletedEvent(final UserTokenEventTypes eventTypes, final int deleted) {
    this(eventTypes, deleted, null);
  }

  public UserTokenDeletedEvent(final UserTokenEventTypes eventTypes, final int deleted, final String username) {
    super(eventTypes);
    this.deleted = deleted;
    this.username = username;
  }

  public int getDeleted() {
    return this.deleted;
  }

  public String getUsername() {
    return username;
  }
}
