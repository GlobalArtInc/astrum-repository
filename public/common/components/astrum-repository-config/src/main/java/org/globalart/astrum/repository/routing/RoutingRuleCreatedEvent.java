
package org.globalart.astrum.repository.routing;

public class RoutingRuleCreatedEvent
    extends RoutingRuleEvent
{
  public RoutingRuleCreatedEvent(final RoutingRule routingRule) {
    super(routingRule);
  }
}
