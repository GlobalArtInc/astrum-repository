
package org.globalart.astrum.security.usertoken.event;

public class UserTokenPurgedEvent
    extends UserTokenDeletedEvent
{
  public UserTokenPurgedEvent(final int deleted) {
    super(UserTokenEventTypes.PURGED, deleted);
  }
}
