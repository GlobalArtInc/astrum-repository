
package org.globalart.astrum.security.usertoken.event;

public class UserTokenEvent
{
  public enum UserTokenEventTypes
  {
    CREATED("Created"),
    DELETED("Deleted"),
    ALL_DELETED("All Deleted"),
    PURGED("Purged"),
    CONFIG_UPDATED("Configuration Changed"),
    ADMIN_CREATED("Admin Created"),
    ADMIN_READ("Admin Read"),
    ADMIN_DELETED("Admin Deleted"),
    ADMIN_LISTED("Admin Listed"),
    ADMIN_FAILED("Admin Operation Failed");

    private final String type;

    UserTokenEventTypes(final String type) {
      this.type = type;
    }

    public String getType() {
      return this.type;
    }
  }

  private final UserTokenEventTypes eventType;

  public UserTokenEvent(final UserTokenEventTypes eventType) {
    this.eventType = eventType;
  }

  public UserTokenEventTypes getEventType() {
    return eventType;
  }
}
