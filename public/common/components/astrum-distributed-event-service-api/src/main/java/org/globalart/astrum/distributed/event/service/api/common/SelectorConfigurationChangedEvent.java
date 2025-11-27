
package org.globalart.astrum.distributed.event.service.api.common;

import org.globalart.astrum.distributed.event.service.api.EventType;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;

/**
 * Indicates that a selector configuration has been changed.
 *
 * @since 3.38
 */
public class SelectorConfigurationChangedEvent
    extends DistributedEventSupport
{
  public static final String NAME = "SelectorConfigurationChangedEvent";

  @JsonCreator
  public SelectorConfigurationChangedEvent(@JsonProperty("eventType") final EventType eventType) {
    super(eventType);
  }
}
