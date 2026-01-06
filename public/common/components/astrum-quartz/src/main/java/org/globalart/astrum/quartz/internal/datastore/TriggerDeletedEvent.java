
package org.globalart.astrum.quartz.internal.datastore;

import org.quartz.TriggerKey;

public class TriggerDeletedEvent
    extends TriggerEventSupport
{
  TriggerDeletedEvent() {
    // deserialization
  }

  public TriggerDeletedEvent(final TriggerKey triggerKey) {
    super(triggerKey);
  }
}
