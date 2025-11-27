
package org.globalart.astrum.distributed.event.service.api.common;

import org.globalart.astrum.common.event.EventWithSource;
import org.globalart.astrum.distributed.event.service.api.EventType;

import static com.google.common.base.Preconditions.checkNotNull;

public abstract class DistributedEventSupport
    extends EventWithSource
{
  private final EventType eventType;

  protected DistributedEventSupport(final EventType eventType) {
    this.eventType = checkNotNull(eventType);
  }

  public EventType getEventType() {
    return eventType;
  }
}
