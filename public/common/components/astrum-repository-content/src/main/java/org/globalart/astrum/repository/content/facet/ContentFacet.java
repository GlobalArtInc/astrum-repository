
package org.globalart.astrum.repository.content.facet;

import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.content.ContentRepository;
import org.globalart.astrum.repository.content.fluent.FluentAssets;
import org.globalart.astrum.repository.content.fluent.FluentAttributes;
import org.globalart.astrum.repository.content.fluent.FluentBlobs;
import org.globalart.astrum.repository.content.fluent.FluentComponents;

/**
 * Repository {@link Facet} for content related metadata.
 *
 * @since 3.21
 */
@Facet.Exposed
public interface ContentFacet
    extends Facet, ContentRepository, FluentAttributes<ContentFacet>
{
  /**
   * Ingest blobs.
   */
  FluentBlobs blobs();

  /**
   * Manage components.
   */
  FluentComponents components();

  /**
   * Manage assets.
   */
  FluentAssets assets();

  /**
   * Invoked when converting a temporary blob into a permanently blob to identify which headers to keep for the
   * permanent blob.
   */
  default boolean shouldKeepBlobHeader(final String blobHeaderName) {
    return false;
  }
}
