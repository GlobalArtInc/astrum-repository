
package org.globalart.astrum.quartz.internal.datastore;

import org.quartz.TriggerKey;

public class TriggerUpdatedEvent
    extends TriggerEventSupport
{
  TriggerUpdatedEvent() {
    // deserialization
  }

  public TriggerUpdatedEvent(final TriggerKey triggerKey) {
    super(triggerKey);
  }
}
