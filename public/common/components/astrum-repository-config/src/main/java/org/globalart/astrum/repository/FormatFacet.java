
package org.globalart.astrum.repository;

import javax.annotation.Nullable;

/**
 * Facet to retrieve the format, or subformat of a repository
 *
 * @since 3.22
 */
@Facet.Exposed
public interface FormatFacet
    extends Facet
{
  Format getFormat();

  @Nullable
  String getSubFormat();

  /**
   * Hook to allow a sub format to be identified
   * 
   * @since 3.24
   */
  void markSubFormat();
}
