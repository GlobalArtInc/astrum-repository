
package org.globalart.astrum.security.usertoken.event;

public class UserTokenConfigChangedEvent
    extends UserTokenEvent
{
  private final boolean enabled;

  private final boolean protectContent;

  private final boolean expirationEnabled;

  private final int expirationDays;

  public UserTokenConfigChangedEvent(
      final boolean enabled,
      final boolean protectContent,
      final boolean expirationEnabled,
      final int expirationDays)
  {
    super(UserTokenEventTypes.CONFIG_UPDATED);
    this.enabled = enabled;
    this.protectContent = protectContent;
    this.expirationEnabled = expirationEnabled;
    this.expirationDays = expirationDays;
  }

  public boolean isEnabled() {
    return enabled;
  }

  public boolean isProtectContent() {
    return protectContent;
  }

  public boolean isExpirationEnabled() {
    return expirationEnabled;
  }

  public int getExpirationDays() {
    return expirationDays;
  }
}
