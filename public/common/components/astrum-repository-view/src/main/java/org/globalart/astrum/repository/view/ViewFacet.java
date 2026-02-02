
package org.globalart.astrum.repository.view;

import org.globalart.astrum.repository.Facet;

/**
 * View facet.
 *
 * @since 3.0
 */
@Facet.Exposed
public interface ViewFacet
    extends Facet
{
  /**
   * Dispatch request to router.
   */
  Response dispatch(Request request) throws Exception;

  /**
   * Dispatch request to router with an existing context to pull attributes from.
   * 
   * @since 3.1
   */
  Response dispatch(Request request, Context context) throws Exception;
}
