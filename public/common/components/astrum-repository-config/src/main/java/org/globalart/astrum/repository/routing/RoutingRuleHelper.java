
package org.globalart.astrum.repository.routing;

import java.util.List;
import java.util.Map;

import org.globalart.astrum.common.entity.EntityId;
import org.globalart.astrum.repository.Repository;

/**
 * @since 3.16
 */
public interface RoutingRuleHelper
{
  /**
   * Determine if the path is allowed if the repository has a routing rule configured.
   *
   * @param repository the repository for the context of this request
   * @param path the path of the component (must include leading slash)
   * @return true if the request is allowed, false if it should be blocked
   */
  boolean isAllowed(final Repository repository, final String path);

  /**
   * Determine if the path is allowed for a routing rule.
   *
   * @param routingRule used to allow or block the request
   * @param path the path of the request
   * @return true if the request is allowed, false if it should be blocked
   */
  boolean isAllowed(final RoutingRule routingRule, final String path);

  /**
   * Determine if the path is allowed by the RoutingRule, does not consider whether the configuration is enabled.
   *
   * @param mode the routing mode to test the path against
   * @param matchers the list of matchers to test the path against
   * @param path the path of the component (must include leading slash)
   * @return true if the request is allowed, false if it should be blocked
   */
  boolean isAllowed(final RoutingMode mode, final List<String> matchers, final String path);

  /**
   * Iterates through all repositories to find which routing rules are assigned
   *
   * @return A map of routing rule ids to a list of repositories that are using them
   */
  Map<EntityId, List<Repository>> calculateAssignedRepositories();

  /**
   * Ensures that the user has the necessary permissions to Read routing rules
   */
  void ensureUserHasPermissionToRead();

  /**
   * Determine if the repository has a routing rule configured.
   *
   * @param repository the repository for the context of this request
   * @return true if the repository has a routing rule configured
   */
  boolean hasRoutingRule(final Repository repository);
}
