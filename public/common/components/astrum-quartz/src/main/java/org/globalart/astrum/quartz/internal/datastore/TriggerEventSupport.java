
package org.globalart.astrum.quartz.internal.datastore;

import org.globalart.astrum.common.event.EventWithSource;

import com.fasterxml.jackson.annotation.JsonIgnore;
import org.quartz.TriggerKey;

public class TriggerEventSupport
    extends EventWithSource
{
  private String triggerGroup;

  private String triggerName;

  protected TriggerEventSupport() {
    // deserialization
  }

  public TriggerEventSupport(final TriggerKey triggerKey) {
    this.triggerGroup = triggerKey.getGroup();
    this.triggerName = triggerKey.getName();
  }

  public String getTriggerGroup() {
    return triggerGroup;
  }

  @JsonIgnore
  public TriggerKey getTriggerKey() {
    return new TriggerKey(triggerName, triggerGroup);
  }

  public String getTriggerName() {
    return triggerName;
  }

  public void setTriggerGroup(final String triggerGroup) {
    this.triggerGroup = triggerGroup;
  }

  public void setTriggerName(final String triggerName) {
    this.triggerName = triggerName;
  }
}
