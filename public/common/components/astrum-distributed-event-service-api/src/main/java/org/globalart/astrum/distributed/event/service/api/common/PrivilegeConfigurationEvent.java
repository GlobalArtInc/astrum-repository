
package org.globalart.astrum.distributed.event.service.api.common;

import org.globalart.astrum.distributed.event.service.api.EventType;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Indicates that a 'Privilege' configuration has been created, updated or deleted.
 *
 * @since 3.38
 */
public class PrivilegeConfigurationEvent
    extends DistributedEventSupport
{
  public static final String NAME = "PrivilegeConfigurationEvent";

  private final String privilegeId;

  @JsonCreator
  public PrivilegeConfigurationEvent(
      @JsonProperty("privilegeId") String privilegeId,
      @JsonProperty("eventType") final EventType eventType)
  {
    super(eventType);
    this.privilegeId = checkNotNull(privilegeId);
  }

  public String getPrivilegeId() {
    return privilegeId;
  }

  @Override
  public String toString() {
    return "PrivilegeConfigurationEvent{" +
        "privilegeId='" + privilegeId + '\'' +
        '}';
  }
}
