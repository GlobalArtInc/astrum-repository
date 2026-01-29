
package org.globalart.astrum.repository.routing;

import static com.google.common.base.Preconditions.checkNotNull;

public class RoutingRuleEvent
{
  private final RoutingRule routingRule;

  public RoutingRuleEvent(final RoutingRule routingRule) {
    this.routingRule = checkNotNull(routingRule);
  }

  public RoutingRule getRoutingRule() {
    return routingRule;
  }
}
