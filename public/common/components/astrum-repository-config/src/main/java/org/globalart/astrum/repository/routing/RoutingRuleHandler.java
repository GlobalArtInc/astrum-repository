
package org.globalart.astrum.repository.routing;

import org.globalart.astrum.repository.recipe.Handler;

/**
 * A handler which validates that the request is allowed by the RoutingRule assigned to the repository.
 *
 * This interface helps bridge to the real implementation which now lives in another module.
 *
 * @since 3.22
 */
public interface RoutingRuleHandler
    extends Handler
{
  // intentionally blank
}
