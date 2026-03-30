
package org.globalart.astrum.repository.npm.internal;

import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Response;

/**
 * npm token management facet.
 *
 * @since 3.0
 */
@Facet.Exposed
public interface NpmTokenFacet
    extends Facet
{
  /**
   * Performs a login for user authenticated in the request (creates token and returns login specific response).
   */
  Response login(Context context);

  /**
   * Performs a log-out for currently authenticated user (deletes the token if found and returns logout specific
   * response).
   */
  Response logout(Context context);
}
