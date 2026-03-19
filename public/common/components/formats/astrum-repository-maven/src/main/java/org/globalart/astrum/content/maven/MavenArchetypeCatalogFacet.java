
package org.globalart.astrum.content.maven;

import java.io.IOException;

import org.globalart.astrum.repository.Facet;

/**
 * Facet for rebuilding maven archetype catalog.
 *
 * @since 3.25
 */
@Facet.Exposed
public interface MavenArchetypeCatalogFacet
    extends Facet
{
  void rebuildArchetypeCatalog() throws IOException;
}
