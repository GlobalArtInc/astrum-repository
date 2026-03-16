
package org.globalart.astrum.content.example;

import java.io.IOException;
import java.util.Optional;

import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Payload;

/**
 * Provides persistent content for an 'example' format.
 *
 * @since 3.24
 */
@Facet.Exposed
public interface ExampleContentFacet
    extends ContentFacet
{
  Optional<Content> get(String path) throws IOException;

  Content put(String path, Payload content) throws IOException;

  boolean delete(String path) throws IOException;
}
