
package org.globalart.astrum.distributed.event.service.api.common;

import org.globalart.astrum.distributed.event.service.api.EventType;

/**
 * Indicates that authorization changed.
 *
 * @since 3.38
 */
public class AuthorizationChangedDistributedEvent
    extends DistributedEventSupport
{
  public static final String NAME = "AuthorizationChangedDistributedEvent";

  public AuthorizationChangedDistributedEvent() {
    super(EventType.CREATED);
  }

  @Override
  public String toString() {
    return NAME + "{" +
        "eventType='" + getEventType() + '\'' +
        "}";
  }
}
