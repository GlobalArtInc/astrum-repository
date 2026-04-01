
package org.globalart.astrum.repository.pypi.internal;

import org.globalart.astrum.repository.Facet;

/**
 * Persistence for PyPI indexes.
 *
 * @since 3.16
 */
@Facet.Exposed
public interface PyPiIndexFacet
    extends Facet
{
  /**
   * Deletes the root index asset
   */
  void deleteRootIndex();

  /**
   * Deletes an index related to a given package
   *
   * @param packageName - packageName to use for finding the index to be removed
   */
  void deleteIndex(String packageName);
}
