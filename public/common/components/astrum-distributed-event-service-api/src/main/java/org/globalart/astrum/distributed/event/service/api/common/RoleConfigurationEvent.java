
package org.globalart.astrum.distributed.event.service.api.common;

import org.globalart.astrum.distributed.event.service.api.EventType;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Indicates that a 'Role' configuration has been created, updated or deleted.
 *
 * @since 3.38
 */
public class RoleConfigurationEvent
    extends DistributedEventSupport
{
  public static final String NAME = "RoleConfigurationEvent";

  private final String roleId;

  @JsonCreator
  public RoleConfigurationEvent(
      @JsonProperty("roleId") String roleId,
      @JsonProperty("eventType") final EventType eventType)
  {
    super(eventType);
    this.roleId = checkNotNull(roleId);
  }

  public String getRoleId() {
    return roleId;
  }

  @Override
  public String toString() {
    return "RoleConfigurationEvent{" +
        "roleId='" + roleId + '\'' +
        '}';
  }
}
