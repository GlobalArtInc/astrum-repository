
package org.globalart.astrum.distributed.event.service.api.common;

import org.globalart.astrum.distributed.event.service.api.EventType;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;

/**
 * Indicates that a ssl certificate has been created or deleted.
 */
public class CertificateDistributedEvent
    extends DistributedEventSupport
{
  public static final String NAME = "CertificateDistributedEvent";

  @JsonCreator
  public CertificateDistributedEvent(@JsonProperty("eventType") final EventType eventType) {
    super(eventType);
  }

  @Override
  public String toString() {
    return NAME + "{" +
        "eventType='" + getEventType() + '\'' +
        "}";
  }
}
