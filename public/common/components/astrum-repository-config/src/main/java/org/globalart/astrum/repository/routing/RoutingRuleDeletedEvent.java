
package org.globalart.astrum.repository.routing;

public class RoutingRuleDeletedEvent
    extends RoutingRuleEvent
{
  public RoutingRuleDeletedEvent(final RoutingRule routingRule) {
    super(routingRule);
  }
}
