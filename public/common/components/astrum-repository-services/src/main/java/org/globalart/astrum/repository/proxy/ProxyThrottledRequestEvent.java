
package org.globalart.astrum.repository.proxy;

import org.globalart.astrum.common.event.Event;

public class ProxyThrottledRequestEvent
    implements Event
{
  private final boolean isBlocked;

  public ProxyThrottledRequestEvent(final boolean isBlocked) {
    this.isBlocked = isBlocked;
  }

  public boolean isBlocked() {
    return isBlocked;
  }
}
