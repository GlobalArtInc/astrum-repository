
package org.globalart.astrum.common.event;

public class HostedThrottledRequestEvent
    implements Event
{
  private final boolean isBlocked;

  public HostedThrottledRequestEvent(final boolean isBlocked) {
    this.isBlocked = isBlocked;
  }

  public boolean isBlocked() {
    return isBlocked;
  }
}
