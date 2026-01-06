
package org.globalart.astrum.quartz.internal.datastore;

import org.quartz.TriggerKey;

public class TriggerCreatedEvent
    extends TriggerEventSupport
{
  TriggerCreatedEvent() {
    // deserialization
  }

  public TriggerCreatedEvent(final TriggerKey triggerKey) {
    super(triggerKey);
  }
}
