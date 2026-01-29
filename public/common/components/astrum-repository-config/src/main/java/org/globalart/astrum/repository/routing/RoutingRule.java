
package org.globalart.astrum.repository.routing;

import java.util.List;

import org.globalart.astrum.common.entity.EntityId;

/**
 * A RoutingRule which can be applied to a repository to block or allow requests depending on the RoutingMode
 *
 * @since 3.16
 */
public interface RoutingRule
{
  String description();

  RoutingRule description(final String description);

  /**
   * The block mode of this RoutingRule, when ACCESS then any request which does not match one of the matchers should be
   * blocked and when BLOCK any request which matches one of the matchers should be blocked.
   */
  RoutingMode mode();

  RoutingRule mode(final RoutingMode mode);

  /**
   * The name of this RoutingRule, must be unique
   */
  String name();

  RoutingRule name(final String name);

  /**
   * @return the list of regex matchers which this RoutingRule uses
   */
  List<String> matchers();

  RoutingRule matchers(final List<String> matchers);

  EntityId id();
}
