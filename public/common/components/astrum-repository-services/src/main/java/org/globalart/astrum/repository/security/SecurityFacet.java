
package org.globalart.astrum.repository.security;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.view.Request;

import org.apache.shiro.authz.AuthorizationException;

/**
 * Security facet.
 *
 * @since 3.0
 */
@Facet.Exposed
public interface SecurityFacet
    extends Facet
{
  /**
   * Ensure the given request is permitted on the repository.
   *
   * @throws AuthorizationException
   */
  void ensurePermitted(Request request);

  /**
   * Ensures that the current user has permission to perform the action implied by the request.
   * <p>
   * This overload accepts an optional request-scoped cache for selector evaluations,
   * which significantly improves performance for scenarios with repeated selector checks
   * (e.g., Docker digest pulls from group repositories).
   * </p>
   *
   * @param request the request to authorize
   * @param selectorCache optional cache for selector evaluation results (null to disable caching)
   * @throws AuthorizationException if not authorized
   */
  void ensurePermitted(Request request, @Nullable SelectorEvaluationCache selectorCache);
}
