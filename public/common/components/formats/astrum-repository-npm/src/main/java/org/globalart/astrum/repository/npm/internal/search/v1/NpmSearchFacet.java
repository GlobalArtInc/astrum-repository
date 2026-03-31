
package org.globalart.astrum.repository.npm.internal.search.v1;

import java.io.IOException;

import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Parameters;

/**
 * Facet for npm V1 search, replacing the existing npm search indexes that are now deprecated.
 *
 * @since 3.7
 */
@Facet.Exposed
public interface NpmSearchFacet
    extends Facet
{
  /**
   * Fetches the v1 search results.
   */
  Content searchV1(final Parameters parameters) throws IOException;
}
