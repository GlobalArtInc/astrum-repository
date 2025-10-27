
package org.globalart.astrum.common.event;

import javax.annotation.Nullable;

import com.fasterxml.jackson.annotation.JsonIgnore;

/**
 * Base class for events which carry origin information
 */
public abstract class EventWithSource
    implements HasLocality
{
  private String remoteNodeId;

  /**
   * @return {@code true} if this is a local event
   */
  @JsonIgnore
  @Override
  public boolean isLocal() {
    return remoteNodeId == null;
  }

  /**
   * @param remoteNodeId the remote node that sent this event; {@code null} if event is local
   */
  public void setRemoteNodeId(@Nullable final String remoteNodeId) {
    this.remoteNodeId = remoteNodeId;
  }

  /**
   * @return the remote node that sent this event; {@code null} if event is local
   */
  @Nullable
  public String getRemoteNodeId() {
    return remoteNodeId;
  }
}
