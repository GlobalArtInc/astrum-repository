
package org.globalart.astrum.repository.routing;

import org.globalart.astrum.common.entity.EntityId;
import org.globalart.astrum.common.event.EventWithSource;

/**
 * @since 3.21
 */
public class RoutingRuleInvalidatedEvent
    extends EventWithSource
{
  private String routingRuleId;

  public RoutingRuleInvalidatedEvent() {
    // required for Jackson deserialization
  }

  public RoutingRuleInvalidatedEvent(final EntityId routingRuleId) {
    this.routingRuleId = routingRuleId.getValue();
  }

  public String getRoutingRuleId() {
    return routingRuleId;
  }

  public void setRoutingRuleId(final String routingRuleId) {
    this.routingRuleId = routingRuleId;
  }
}
