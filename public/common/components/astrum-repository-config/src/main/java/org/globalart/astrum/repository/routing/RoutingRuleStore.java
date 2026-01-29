
package org.globalart.astrum.repository.routing;

import java.util.List;

import javax.annotation.Nullable;

/**
 * @since 3.16
 */
public interface RoutingRuleStore
{
  RoutingRule create(RoutingRule rule);

  void update(RoutingRule rule);

  void delete(RoutingRule rule);

  List<RoutingRule> list();

  @Nullable
  RoutingRule getByName(String name);

  @Nullable
  RoutingRule getById(String id);

  RoutingRule newRoutingRule();
}
