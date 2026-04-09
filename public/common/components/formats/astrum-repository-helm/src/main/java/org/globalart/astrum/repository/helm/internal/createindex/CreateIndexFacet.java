
package org.globalart.astrum.repository.helm.internal.createindex;

import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.Facet.Exposed;

/**
 * Facet interface for rebuilding Helm index.yaml files
 *
 * @since 3.28
 */
@Exposed
public interface CreateIndexFacet
    extends Facet
{
  /**
   * Mark the helm index yaml as invalidated such that it will be rebuilt after waiting for a configured amount of time
   * to prevent unnecessary successive rebuilds of the metadata.
   */
  void invalidateIndex();
}
