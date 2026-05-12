
package org.globalart.astrum.rapture.internal.settings;

import org.globalart.astrum.common.event.EventWithSource;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;

/**
 * Event published when session timeout configuration changes.
 * Cluster-distributable: will propagate to all nodes in HA deployments.
 */
public class SessionTimeoutChangedEvent
    extends EventWithSource
{
  private final int timeoutMinutes;

  @JsonCreator
  public SessionTimeoutChangedEvent(@JsonProperty("timeoutMinutes") final int timeoutMinutes) {
    this.timeoutMinutes = timeoutMinutes;
  }

  public int getTimeoutMinutes() {
    return timeoutMinutes;
  }
}
